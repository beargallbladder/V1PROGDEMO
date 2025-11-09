use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dealer {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub zip_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub zip_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub dealer: DealerResponse,
}

#[derive(Debug, Serialize)]
pub struct DealerResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub zip_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Dealer> for DealerResponse {
    fn from(dealer: Dealer) -> Self {
        DealerResponse {
            id: dealer.id,
            name: dealer.name,
            email: dealer.email,
            zip_code: dealer.zip_code,
            created_at: dealer.created_at,
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct Upload {
    pub id: i32,
    pub dealer_id: i32,
    pub filename: String,
    pub file_path: String,
    pub uploaded_at: DateTime<Utc>,
    pub status: String,
    pub row_count: i32,
    pub processed_count: i32,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Vehicle {
    pub id: i32,
    pub upload_id: i32,
    pub dealer_id: i32,
    pub vin: String,
    pub warranty_exp_date: Option<NaiveDate>,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub customer_zip: Option<String>,
    pub last_service_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VehicleInput {
    pub vin: String,
    pub warranty_exp_date: Option<String>,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub customer_zip: Option<String>,
    pub last_service_date: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ScoredLead {
    pub id: i32,
    pub vehicle_id: i32,
    pub upload_id: i32,
    pub urgency_score: f32,
    pub stressor_score: f32,
    pub warranty_score: f32,
    pub susceptibility_score: f32,
    pub telematic_score: f32,
    pub has_telematic: bool,
    pub stressor_type: Option<String>,
    pub why_now: String,
    pub call_by_date: NaiveDate,
    pub suggested_script: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ScoredLeadWithVehicle {
    #[serde(flatten)]
    pub lead: ScoredLead,
    pub vehicle: Vehicle,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

