#!/bin/bash

# Render API Key
API_KEY="rnd_3mjr1vyK3ayU9xDZkZ8PytUiizpQ"
BASE_URL="https://api.render.com/v1"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Deploying Stressor Leads to Render...${NC}\n"

# Step 1: Create PostgreSQL Database
echo -e "${YELLOW}Step 1: Creating PostgreSQL database...${NC}"
DB_RESPONSE=$(curl -s -X POST "${BASE_URL}/databases" \
  -H "Authorization: Bearer ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "stressor-leads-db",
    "databaseName": "stressor_leads",
    "user": "stressor_leads",
    "plan": "free",
    "region": "oregon"
  }')

DB_ID=$(echo $DB_RESPONSE | grep -o '"id":"[^"]*' | cut -d'"' -f4)
DB_CONNECTION_STRING=$(echo $DB_RESPONSE | grep -o '"connectionString":"[^"]*' | cut -d'"' -f4)

if [ -z "$DB_ID" ]; then
  echo "❌ Failed to create database. Response: $DB_RESPONSE"
  exit 1
fi

echo -e "${GREEN}✅ Database created! ID: $DB_ID${NC}"
echo -e "${GREEN}   Connection string: $DB_CONNECTION_STRING${NC}\n"

# Step 2: Create Web Service
echo -e "${YELLOW}Step 2: Creating web service...${NC}"

# Generate JWT secret
JWT_SECRET=$(openssl rand -hex 32)

WEB_SERVICE_RESPONSE=$(curl -s -X POST "${BASE_URL}/services" \
  -H "Authorization: Bearer ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"stressor-leads-api\",
    \"type\": \"web_service\",
    \"repo\": \"https://github.com/beargallbladder/V1PROGDEMO\",
    \"branch\": \"main\",
    \"rootDir\": \"stressor-leads\",
    \"env\": \"rust\",
    \"buildCommand\": \"cargo build --release\",
    \"startCommand\": \"./target/release/stressor-leads\",
    \"plan\": \"free\",
    \"region\": \"oregon\",
    \"envVars\": [
      {
        \"key\": \"DATABASE_URL\",
        \"value\": \"${DB_CONNECTION_STRING}\"
      },
      {
        \"key\": \"JWT_SECRET\",
        \"value\": \"${JWT_SECRET}\"
      },
      {
        \"key\": \"PORT\",
        \"value\": \"10000\"
      }
    ],
    \"healthCheckPath\": \"/api/health\"
  }")

WEB_SERVICE_ID=$(echo $WEB_SERVICE_RESPONSE | grep -o '"id":"[^"]*' | cut -d'"' -f4)
WEB_SERVICE_URL=$(echo $WEB_SERVICE_RESPONSE | grep -o '"serviceUrl":"[^"]*' | cut -d'"' -f4)

if [ -z "$WEB_SERVICE_ID" ]; then
  echo "❌ Failed to create web service. Response: $WEB_SERVICE_RESPONSE"
  exit 1
fi

echo -e "${GREEN}✅ Web service created! ID: $WEB_SERVICE_ID${NC}"
echo -e "${GREEN}   Service URL: $WEB_SERVICE_URL${NC}\n"

echo -e "${GREEN}🎉 Deployment initiated!${NC}"
echo -e "${YELLOW}Note: It may take a few minutes for the service to build and deploy.${NC}"
echo -e "${YELLOW}Monitor progress at: https://dashboard.render.com${NC}\n"
echo -e "${GREEN}Your API will be available at: $WEB_SERVICE_URL${NC}"
echo -e "${GREEN}Health check: $WEB_SERVICE_URL/api/health${NC}\n"
echo -e "${YELLOW}Next steps:${NC}"
echo -e "1. Wait for deployment to complete"
echo -e "2. Deploy frontend to Vercel"
echo -e "3. Set FRONTEND_URL in Render dashboard to your Vercel URL"
echo -e "4. Redeploy backend service"

