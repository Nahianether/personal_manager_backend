use axum::{
    routing::{get, post, put, delete},
    Router,
    http::{Method, HeaderValue},
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;

mod models;
mod handlers;
mod services;
mod middleware;
mod utils;

use handlers::{
    account::{create_account, get_accounts, get_account, update_account, delete_account},
    // category::{create_category, get_categories, get_category, update_category, delete_category},
    // transaction::{create_transaction, get_transactions, get_transaction, update_transaction, delete_transaction},
    // liability::{create_liability, get_liabilities, get_liability, update_liability, delete_liability},
    // loan::{create_loan, get_loans, get_loan, update_loan, delete_loan},
    auth::{signup, login, signin},
    user_data::{get_user_accounts, get_user_transactions, get_user_loans, get_user_liabilities},
};

#[tokio::main]
async fn main() {
    // Initialize logger with different levels
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    log::info!("🚀 Starting Personal Manager Backend Server...");
    log::info!("📊 Log level: {}", log::max_level());
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./personal_manager.db".to_string());
    log::info!("🗄️  Initializing database: {}", database_url);
    
    let pool = services::database::init_db(&database_url).await.expect("Failed to initialize database");
    
    // Create tables
    log::info!("🔧 Creating database tables...");
    services::database::create_tables(&pool).await.expect("Failed to create tables");

    // Configure CORS for Flutter development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        // Root route
        .route("/", get(|| async { 
            serde_json::json!({
                "message": "Personal Manager Backend API",
                "version": "1.0.0",
                "endpoints": {
                    "auth": {
                        "signup": "/auth/signup",
                        "login": "/auth/login",
                        "signin": "/auth/signin"
                    },
                    "user_data": {
                        "accounts": "/api/accounts",
                        "transactions": "/api/transactions",
                        "loans": "/api/loans",
                        "liabilities": "/api/liabilities"
                    },
                    "accounts": "/accounts",
                    "categories": "/categories", 
                    "transactions": "/transactions",
                    "liabilities": "/liabilities",
                    "loans": "/loans",
                    "health": "/health"
                }
            }).to_string()
        }))
        
        // Authentication routes
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/signin", post(signin))
        
        // User-specific API routes (requires authentication)
        .route("/api/accounts", get(get_user_accounts))
        .route("/api/transactions", get(get_user_transactions))
        .route("/api/loans", get(get_user_loans))
        .route("/api/liabilities", get(get_user_liabilities))
        
        // Account routes with authentication
        .route("/accounts", post(create_account).get(get_accounts))
        .route("/accounts/:id", get(get_account).put(update_account).delete(delete_account))
        // .route("/categories", post(create_category).get(get_categories))
        // .route("/categories/:id", get(get_category).put(update_category).delete(delete_category))
        // .route("/transactions", post(create_transaction).get(get_transactions))
        // .route("/transactions/:id", get(get_transaction).put(update_transaction).delete(delete_transaction))
        // .route("/liabilities", post(create_liability).get(get_liabilities))
        // .route("/liabilities/:id", get(get_liability).put(update_liability).delete(delete_liability))
        // .route("/loans", post(create_loan).get(get_loans))
        // .route("/loans/:id", get(get_loan).put(update_loan).delete(delete_loan))
        
        // Health check
        .route("/health", get(|| async { "OK" }))
        
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server starting...");
    println!("📡 Server running on http://localhost:3000");
    println!("📡 Server running on http://0.0.0.0:3000");
    println!("📋 Available endpoints:");
    println!("   GET  /              - API info");
    println!("   GET  /health        - Health check");
    println!("   GET  /accounts      - Get all accounts");
    println!("   POST /accounts      - Create account");
    println!("   🌐 CORS enabled for all origins");
    println!("✅ Ready to accept connections!");
    
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}