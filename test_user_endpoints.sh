#!/bin/bash

# Test script for user-specific endpoints
BASE_URL="http://localhost:3000"

echo "=== Testing User-Specific Endpoints ==="

# 1. Create a test user
echo "1. Creating test user..."
SIGNUP_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "test@example.com", "password": "password123"}')

echo "Signup response: $SIGNUP_RESPONSE"

# Extract token from response
TOKEN=$(echo "$SIGNUP_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
  echo "Failed to get token from signup response"
  exit 1
fi

echo "Token: $TOKEN"

# 2. Test user accounts endpoint
echo -e "\n2. Testing /api/accounts endpoint..."
curl -s -X GET "$BASE_URL/api/accounts" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" | jq .

# 3. Test user transactions endpoint
echo -e "\n3. Testing /api/transactions endpoint..."
curl -s -X GET "$BASE_URL/api/transactions" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" | jq .

# 4. Test user loans endpoint
echo -e "\n4. Testing /api/loans endpoint..."
curl -s -X GET "$BASE_URL/api/loans" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" | jq .

# 5. Test user liabilities endpoint
echo -e "\n5. Testing /api/liabilities endpoint..."
curl -s -X GET "$BASE_URL/api/liabilities" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" | jq .

# 6. Test without authentication
echo -e "\n6. Testing without authentication (should fail)..."
curl -s -X GET "$BASE_URL/api/accounts" \
  -H "Content-Type: application/json" | jq .

echo -e "\n=== Tests completed ==="