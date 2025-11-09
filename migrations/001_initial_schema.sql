-- Initial database schema for dealers, uploads, vehicles, and scored_leads

CREATE TABLE IF NOT EXISTS dealers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    zip_code VARCHAR(10),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dealers_email ON dealers(email);

CREATE TABLE IF NOT EXISTS uploads (
    id SERIAL PRIMARY KEY,
    dealer_id INTEGER NOT NULL REFERENCES dealers(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    uploaded_at TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(50) DEFAULT 'processing',
    row_count INTEGER DEFAULT 0,
    processed_count INTEGER DEFAULT 0,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_uploads_dealer ON uploads(dealer_id);
CREATE INDEX IF NOT EXISTS idx_uploads_status ON uploads(status);

CREATE TABLE IF NOT EXISTS vehicles (
    id SERIAL PRIMARY KEY,
    upload_id INTEGER NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
    dealer_id INTEGER NOT NULL REFERENCES dealers(id) ON DELETE CASCADE,
    vin VARCHAR(17) NOT NULL,
    warranty_exp_date DATE,
    customer_name VARCHAR(255) NOT NULL,
    customer_phone VARCHAR(20) NOT NULL,
    customer_email VARCHAR(255),
    customer_zip VARCHAR(10),
    last_service_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vehicles_vin ON vehicles(vin);
CREATE INDEX IF NOT EXISTS idx_vehicles_upload ON vehicles(upload_id);
CREATE INDEX IF NOT EXISTS idx_vehicles_dealer ON vehicles(dealer_id);

CREATE TABLE IF NOT EXISTS scored_leads (
    id SERIAL PRIMARY KEY,
    vehicle_id INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    upload_id INTEGER NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
    urgency_score REAL NOT NULL,
    stressor_score REAL NOT NULL,
    warranty_score REAL NOT NULL,
    susceptibility_score REAL NOT NULL,
    telematic_score REAL NOT NULL,
    has_telematic BOOLEAN DEFAULT FALSE,
    stressor_type VARCHAR(50),
    why_now TEXT NOT NULL,
    call_by_date DATE NOT NULL,
    suggested_script TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scored_leads_vehicle ON scored_leads(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_scored_leads_upload ON scored_leads(upload_id);
CREATE INDEX IF NOT EXISTS idx_scored_leads_score ON scored_leads(urgency_score DESC);

