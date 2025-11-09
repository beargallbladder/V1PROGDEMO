use crate::auth::{create_token, hash_password, verify_password, verify_token};
use crate::models::*;
use crate::scoring::calculate_lead_scores;
use axum::{
    extract::{Multipart, Path as AxumPath, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::fs;
use std::path::Path as StdPath;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

// Helper function to get dealer from token
async fn get_dealer_from_request(
    headers: &HeaderMap,
    pool: &PgPool,
) -> Result<Dealer, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let dealer = sqlx::query_as::<_, Dealer>(
        "SELECT id, name, email, password_hash, zip_code, created_at FROM dealers WHERE id = $1"
    )
    .bind(claims.dealer_id)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(dealer)
}

// Dealer handlers
pub async fn register_dealer(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<DealerResponse>>, StatusCode> {
    // Check if email already exists
    let existing = sqlx::query_as::<_, Dealer>(
        "SELECT id, name, email, password_hash, zip_code, created_at FROM dealers WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None::<DealerResponse>,
            error: Some("Email already registered".to_string()),
        }));
    }

    // Hash password
    let password_hash = hash_password(&payload.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert dealer
    let dealer = sqlx::query_as::<_, Dealer>(
        "INSERT INTO dealers (name, email, password_hash, zip_code) VALUES ($1, $2, $3, $4) RETURNING id, name, email, password_hash, zip_code, created_at"
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.zip_code)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(DealerResponse::from(dealer))))
}

pub async fn login_dealer(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    let dealer = sqlx::query_as::<_, Dealer>(
        "SELECT id, name, email, password_hash, zip_code, created_at FROM dealers WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dealer = dealer.ok_or(StatusCode::UNAUTHORIZED)?;

    let valid = verify_password(&payload.password, &dealer.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !valid {
        return Ok(Json(ApiResponse {
            success: false,
            data: None::<LoginResponse>,
            error: Some("Invalid credentials".to_string()),
        }));
    }

    let token = create_token(dealer.id, &dealer.email)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        dealer: DealerResponse::from(dealer),
    })))
}

pub async fn get_dealer_profile(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<DealerResponse>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    Ok(Json(ApiResponse::success(DealerResponse::from(dealer))))
}

// Upload handlers
pub async fn upload_file(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<Upload>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let mut filename = None;
    let mut file_data = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            if let Ok(data) = field.bytes().await {
                file_data = data.to_vec();
            }
        }
    }

    let filename = filename.ok_or(StatusCode::BAD_REQUEST)?;
    
    // Create uploads directory if it doesn't exist
    let uploads_dir = StdPath::new("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Save file
    let file_path = uploads_dir.join(&filename);
    let mut file = File::create(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.write_all(&file_data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_path_str = file_path.to_string_lossy().to_string();

    // Create upload record
    let upload = sqlx::query_as::<_, Upload>(
        "INSERT INTO uploads (dealer_id, filename, file_path, status) VALUES ($1, $2, $3, 'processing') RETURNING id, dealer_id, filename, file_path, uploaded_at, status, row_count, processed_count, error_message"
    )
    .bind(dealer.id)
    .bind(&filename)
    .bind(&file_path_str)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Process file asynchronously
    let pool_clone = pool.clone();
    let upload_id = upload.id;
    tokio::spawn(async move {
        if let Err(e) = process_upload_file(upload_id, &file_path_str, dealer.id, &pool_clone).await {
            eprintln!("Error processing upload {}: {}", upload_id, e);
            let _ = sqlx::query("UPDATE uploads SET status = 'error', error_message = $1 WHERE id = $2")
                .bind(e.to_string())
                .bind(upload_id)
                .execute(&pool_clone)
                .await;
        }
    });

    Ok(Json(ApiResponse::success(upload)))
}

async fn process_upload_file(
    upload_id: i32,
    file_path: &str,
    dealer_id: i32,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;

    let mut row_count = 0;
    let mut processed_count = 0;

    // Read CSV and process each row
    for result in reader.records() {
        let record = result?;
        row_count += 1;

        // Parse vehicle data from CSV
        // Expected columns: vin, warranty_exp_date, customer_name, customer_phone, customer_email, customer_zip, last_service_date
        if record.len() >= 4 {
            let vin = record.get(0).unwrap_or("").to_string();
            let warranty_exp_date_str = record.get(1).unwrap_or("");
            let customer_name = record.get(2).unwrap_or("").to_string();
            let customer_phone = record.get(3).unwrap_or("").to_string();
            let customer_email = record.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string());
            let customer_zip = record.get(5).filter(|s| !s.is_empty()).map(|s| s.to_string());
            let last_service_date_str = record.get(6).unwrap_or("");

            let warranty_exp_date = warranty_exp_date_str
                .parse::<NaiveDate>()
                .ok();
            let last_service_date = last_service_date_str
                .parse::<NaiveDate>()
                .ok();

            // Insert vehicle
            let vehicle = sqlx::query_as::<_, Vehicle>(
                "INSERT INTO vehicles (upload_id, dealer_id, vin, warranty_exp_date, customer_name, customer_phone, customer_email, customer_zip, last_service_date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id, upload_id, dealer_id, vin, warranty_exp_date, customer_name, customer_phone, customer_email, customer_zip, last_service_date, created_at"
            )
            .bind(upload_id)
            .bind(dealer_id)
            .bind(&vin)
            .bind(&warranty_exp_date)
            .bind(&customer_name)
            .bind(&customer_phone)
            .bind(&customer_email)
            .bind(&customer_zip)
            .bind(&last_service_date)
            .fetch_one(pool)
            .await?;

            // Calculate scores
            let scores = calculate_lead_scores(&vehicle);

            // Insert scored lead
            sqlx::query(
                "INSERT INTO scored_leads (vehicle_id, upload_id, urgency_score, stressor_score, warranty_score, susceptibility_score, telematic_score, has_telematic, stressor_type, why_now, call_by_date, suggested_script) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
            )
            .bind(vehicle.id)
            .bind(upload_id)
            .bind(scores.urgency_score)
            .bind(scores.stressor_score)
            .bind(scores.warranty_score)
            .bind(scores.susceptibility_score)
            .bind(scores.telematic_score)
            .bind(scores.has_telematic)
            .bind(&scores.stressor_type)
            .bind(&scores.why_now)
            .bind(scores.call_by_date)
            .bind(&scores.suggested_script)
            .execute(pool)
            .await?;

            processed_count += 1;
        }
    }

    // Update upload status
    sqlx::query("UPDATE uploads SET status = 'completed', row_count = $1, processed_count = $2 WHERE id = $3")
        .bind(row_count)
        .bind(processed_count)
        .bind(upload_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn list_uploads(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<Upload>>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let uploads = sqlx::query_as::<_, Upload>(
        "SELECT id, dealer_id, filename, file_path, uploaded_at, status, row_count, processed_count, error_message FROM uploads WHERE dealer_id = $1 ORDER BY uploaded_at DESC"
    )
    .bind(dealer.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(uploads)))
}

pub async fn get_upload(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    AxumPath(upload_id): AxumPath<i32>,
) -> Result<Json<ApiResponse<Upload>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let upload = sqlx::query_as::<_, Upload>(
        "SELECT id, dealer_id, filename, file_path, uploaded_at, status, row_count, processed_count, error_message FROM uploads WHERE id = $1 AND dealer_id = $2"
    )
    .bind(upload_id)
    .bind(dealer.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let upload = upload.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(ApiResponse::success(upload)))
}

// Vehicle handlers
pub async fn list_vehicles(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Vehicle>>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let upload_id = params.get("upload_id").and_then(|s| s.parse::<i32>().ok());
    
    let vehicles = if let Some(upload_id) = upload_id {
        sqlx::query_as::<_, Vehicle>(
            "SELECT id, upload_id, dealer_id, vin, warranty_exp_date, customer_name, customer_phone, customer_email, customer_zip, last_service_date, created_at FROM vehicles WHERE dealer_id = $1 AND upload_id = $2 ORDER BY created_at DESC"
        )
        .bind(dealer.id)
        .bind(upload_id)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, Vehicle>(
            "SELECT id, upload_id, dealer_id, vin, warranty_exp_date, customer_name, customer_phone, customer_email, customer_zip, last_service_date, created_at FROM vehicles WHERE dealer_id = $1 ORDER BY created_at DESC LIMIT 100"
        )
        .bind(dealer.id)
        .fetch_all(&pool)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(vehicles)))
}

pub async fn get_vehicle(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    AxumPath(vehicle_id): AxumPath<i32>,
) -> Result<Json<ApiResponse<Vehicle>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let vehicle = sqlx::query_as::<_, Vehicle>(
        "SELECT id, upload_id, dealer_id, vin, warranty_exp_date, customer_name, customer_phone, customer_email, customer_zip, last_service_date, created_at FROM vehicles WHERE id = $1 AND dealer_id = $2"
    )
    .bind(vehicle_id)
    .bind(dealer.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let vehicle = vehicle.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(ApiResponse::success(vehicle)))
}

// Scored leads handlers
pub async fn list_scored_leads(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<ScoredLeadWithVehicle>>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let upload_id = params.get("upload_id").and_then(|s| s.parse::<i32>().ok());
    let min_score = params.get("min_score").and_then(|s| s.parse::<f32>().ok());
    let limit = params.get("limit").and_then(|s| s.parse::<i32>().ok()).unwrap_or(100);

    let mut query = "SELECT sl.id, sl.vehicle_id, sl.upload_id, sl.urgency_score, sl.stressor_score, sl.warranty_score, sl.susceptibility_score, sl.telematic_score, sl.has_telematic, sl.stressor_type, sl.why_now, sl.call_by_date, sl.suggested_script, sl.created_at, v.id, v.upload_id, v.dealer_id, v.vin, v.warranty_exp_date, v.customer_name, v.customer_phone, v.customer_email, v.customer_zip, v.last_service_date, v.created_at FROM scored_leads sl JOIN vehicles v ON sl.vehicle_id = v.id WHERE v.dealer_id = $1".to_string();

    let mut bind_count = 1;
    if upload_id.is_some() {
        bind_count += 1;
        query.push_str(&format!(" AND sl.upload_id = ${}", bind_count));
    }
    if min_score.is_some() {
        bind_count += 1;
        query.push_str(&format!(" AND sl.urgency_score >= ${}", bind_count));
    }
    query.push_str(" ORDER BY sl.urgency_score DESC");
    query.push_str(&format!(" LIMIT ${}", bind_count + 1));

    let mut query_builder = sqlx::query(&query).bind(dealer.id);
    if let Some(upload_id_val) = upload_id {
        query_builder = query_builder.bind(upload_id_val);
    }
    if let Some(min_score) = min_score {
        query_builder = query_builder.bind(min_score);
    }
    query_builder = query_builder.bind(limit);

    let rows = query_builder
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut leads = Vec::new();
    for row in rows {
        let lead = ScoredLead {
            id: row.get(0),
            vehicle_id: row.get(1),
            upload_id: row.get(2),
            urgency_score: row.get(3),
            stressor_score: row.get(4),
            warranty_score: row.get(5),
            susceptibility_score: row.get(6),
            telematic_score: row.get(7),
            has_telematic: row.get(8),
            stressor_type: row.get(9),
            why_now: row.get(10),
            call_by_date: row.get(11),
            suggested_script: row.get(12),
            created_at: row.get(13),
        };
        let vehicle = Vehicle {
            id: row.get(14),
            upload_id: row.get(15),
            dealer_id: row.get(16),
            vin: row.get(17),
            warranty_exp_date: row.get(18),
            customer_name: row.get(19),
            customer_phone: row.get(20),
            customer_email: row.get(21),
            customer_zip: row.get(22),
            last_service_date: row.get(23),
            created_at: row.get(24),
        };
        leads.push(ScoredLeadWithVehicle { lead, vehicle });
    }

    Ok(Json(ApiResponse::success(leads)))
}

pub async fn get_scored_lead(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    AxumPath(lead_id): AxumPath<i32>,
) -> Result<Json<ApiResponse<ScoredLeadWithVehicle>>, StatusCode> {
    let dealer = get_dealer_from_request(&headers, &pool).await?;
    let row = sqlx::query(
        "SELECT sl.id, sl.vehicle_id, sl.upload_id, sl.urgency_score, sl.stressor_score, sl.warranty_score, sl.susceptibility_score, sl.telematic_score, sl.has_telematic, sl.stressor_type, sl.why_now, sl.call_by_date, sl.suggested_script, sl.created_at, v.id, v.upload_id, v.dealer_id, v.vin, v.warranty_exp_date, v.customer_name, v.customer_phone, v.customer_email, v.customer_zip, v.last_service_date, v.created_at FROM scored_leads sl JOIN vehicles v ON sl.vehicle_id = v.id WHERE sl.id = $1 AND v.dealer_id = $2"
    )
    .bind(lead_id)
    .bind(dealer.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;

    let lead = ScoredLead {
        id: row.get(0),
        vehicle_id: row.get(1),
        upload_id: row.get(2),
        urgency_score: row.get(3),
        stressor_score: row.get(4),
        warranty_score: row.get(5),
        susceptibility_score: row.get(6),
        telematic_score: row.get(7),
        has_telematic: row.get(8),
        stressor_type: row.get(9),
        why_now: row.get(10),
        call_by_date: row.get(11),
        suggested_script: row.get(12),
        created_at: row.get(13),
    };
    let vehicle = Vehicle {
        id: row.get(14),
        upload_id: row.get(15),
        dealer_id: row.get(16),
        vin: row.get(17),
        warranty_exp_date: row.get(18),
        customer_name: row.get(19),
        customer_phone: row.get(20),
        customer_email: row.get(21),
        customer_zip: row.get(22),
        last_service_date: row.get(23),
        created_at: row.get(24),
    };

    Ok(Json(ApiResponse::success(ScoredLeadWithVehicle { lead, vehicle })))
}

