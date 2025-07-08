use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

mod models;
mod handlers;
mod services;
mod middleware;
mod utils;

use handlers::{
    account::{create_account, get_accounts, get_account, update_account, delete_account},
    category::{create_category, get_categories, get_category, update_category, delete_category},
    transaction::{create_transaction, get_transactions, get_transaction, update_transaction, delete_transaction},
    liability::{create_liability, get_liabilities, get_liability, update_liability, delete_liability},
    loan::{create_loan, get_loans, get_loan, update_loan, delete_loan},
};

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./personal_manager.db".to_string());
    let pool = services::database::init_db(&database_url).await.expect("Failed to initialize database");
    
    // Create tables
    services::database::create_tables(&pool).await.expect("Failed to create tables");

    let app = Router::new()
        // Account routes
        .route("/accounts", post(create_account).get(get_accounts))
        .route("/accounts/:id", get(get_account).put(update_account).delete(delete_account))
        
        // Category routes
        .route("/categories", post(create_category).get(get_categories))
        .route("/categories/:id", get(get_category).put(update_category).delete(delete_category))
        
        // Transaction routes
        .route("/transactions", post(create_transaction).get(get_transactions))
        .route("/transactions/:id", get(get_transaction).put(update_transaction).delete(delete_transaction))
        
        // Liability routes
        .route("/liabilities", post(create_liability).get(get_liabilities))
        .route("/liabilities/:id", get(get_liability).put(update_liability).delete(delete_liability))
        
        // Loan routes
        .route("/loans", post(create_loan).get(get_loans))
        .route("/loans/:id", get(get_loan).put(update_loan).delete(delete_loan))
        
        // Health check
        .route("/health", get(|| async { "OK" }))
        
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://{}", addr);
    
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}