# Stressor Leads API

A high-performance REST API built with Rust and Axum for managing dealer leads, vehicle data, and lead scoring.

## Features

- **Dealer Authentication**: JWT-based authentication with bcrypt password hashing
- **File Upload**: CSV file upload and processing for vehicle data
- **Lead Scoring**: Automated scoring algorithm that calculates urgency, stressor, warranty, susceptibility, and telematic scores
- **RESTful API**: Complete CRUD operations for dealers, uploads, vehicles, and scored leads
- **PostgreSQL Database**: Robust database schema with proper indexes and foreign keys
- **Async Processing**: Background processing of uploaded files

## Prerequisites

- Rust (latest stable version)
- PostgreSQL 16+
- Cargo (comes with Rust)

## Setup

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install PostgreSQL** (macOS):
   ```bash
   brew install postgresql@16
   brew services start postgresql@16
   ```

3. **Create the database**:
   ```bash
   createdb stressor_leads
   ```

4. **Set environment variables** (optional, create `.env` file):
   ```bash
   DATABASE_URL=postgresql://localhost/stressor_leads
   JWT_SECRET=your-secret-key-change-in-production
   PORT=3000
   ```

5. **Run the application**:
   ```bash
   cargo run --release
   ```

The server will start on `http://localhost:3000` (or the port specified in `PORT`).

## API Endpoints

### Public Endpoints

- `GET /api/health` - Health check
- `POST /api/dealers/register` - Register a new dealer
- `POST /api/dealers/login` - Login and get JWT token

### Protected Endpoints (Require Bearer Token)

- `GET /api/dealers/me` - Get current dealer profile
- `POST /api/uploads` - Upload a CSV file
- `GET /api/uploads` - List all uploads
- `GET /api/uploads/:id` - Get upload details
- `GET /api/vehicles` - List vehicles (optional: `?upload_id=1`)
- `GET /api/vehicles/:id` - Get vehicle details
- `GET /api/scored-leads` - List scored leads (optional: `?upload_id=1&min_score=0.5&limit=100`)
- `GET /api/scored-leads/:id` - Get scored lead details

## CSV File Format

Upload CSV files with the following columns:
- `vin` - Vehicle Identification Number
- `warranty_exp_date` - Warranty expiration date (YYYY-MM-DD)
- `customer_name` - Customer name
- `customer_phone` - Customer phone number
- `customer_email` - Customer email (optional)
- `customer_zip` - Customer zip code (optional)
- `last_service_date` - Last service date (YYYY-MM-DD, optional)

## Lead Scoring Algorithm

The system calculates multiple scores:

- **Urgency Score**: Weighted combination of all factors
- **Stressor Score**: Based on warranty expiration and service history
- **Warranty Score**: Higher if warranty is expiring soon (within 90 days)
- **Susceptibility Score**: Based on customer data completeness
- **Telematic Score**: Based on telematic data availability

Each lead includes:
- `why_now`: Explanation of why the customer should be contacted
- `call_by_date`: Recommended date to call
- `suggested_script`: Suggested conversation script

## Project Structure

```
stressor-leads/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs             # Library root
│   ├── auth.rs            # Authentication utilities
│   ├── db.rs              # Database connection and migrations
│   ├── handlers.rs        # API route handlers
│   ├── models.rs          # Data models
│   └── scoring.rs         # Lead scoring algorithm
├── migrations/
│   └── 001_initial_schema.sql
└── Cargo.toml
```

## Development

Run in development mode:
```bash
cargo run
```

Run tests:
```bash
cargo test
```

Build for production:
```bash
cargo build --release
```

## Deployment

1. Set environment variables on your deployment platform
2. Ensure PostgreSQL is accessible
3. Run migrations automatically on startup
4. Build and run the release binary

## License

MIT

