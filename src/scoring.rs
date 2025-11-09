use crate::models::Vehicle;
use chrono::{NaiveDate, Utc};

pub struct LeadScores {
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
}

pub fn calculate_lead_scores(vehicle: &Vehicle) -> LeadScores {
    let today = Utc::now().date_naive();
    
    // Warranty score: Higher if warranty is expiring soon (within 90 days)
    let warranty_score = vehicle.warranty_exp_date
        .map(|exp_date| {
            let days_until_expiry = (exp_date - today).num_days();
            if days_until_expiry < 0 {
                0.0 // Warranty already expired
            } else if days_until_expiry <= 30 {
                1.0 // Expiring very soon
            } else if days_until_expiry <= 60 {
                0.8
            } else if days_until_expiry <= 90 {
                0.6
            } else {
                0.3 // Still has time
            }
        })
        .unwrap_or(0.0);

    // Last service date score: Higher if service was long ago
    let service_score = vehicle.last_service_date
        .map(|last_service| {
            let days_since_service = (today - last_service).num_days();
            if days_since_service > 365 {
                0.9 // Over a year since service
            } else if days_since_service > 180 {
                0.7
            } else if days_since_service > 90 {
                0.5
            } else {
                0.2 // Recently serviced
            }
        })
        .unwrap_or(0.8); // No service record = high score

    // Stressor score: Combination of warranty and service
    let stressor_score = (warranty_score * 0.6_f32 + service_score * 0.4_f32).min(1.0_f32);
    
    // Determine stressor type
    let stressor_type = if warranty_score > 0.7 {
        Some("Warranty Expiring".to_string())
    } else if service_score > 0.7 {
        Some("Service Overdue".to_string())
    } else if warranty_score > 0.5 && service_score > 0.5 {
        Some("Multiple Concerns".to_string())
    } else {
        Some("Maintenance Reminder".to_string())
    };

    // Susceptibility score: Based on customer data completeness
    let susceptibility_score = if vehicle.customer_email.is_some() && vehicle.customer_zip.is_some() {
        0.8 // Good contact info
    } else if vehicle.customer_email.is_some() || vehicle.customer_zip.is_some() {
        0.5
    } else {
        0.3
    };

    // Telematic score: Placeholder (would check actual telematic data)
    let has_telematic = false; // Would check actual data
    let telematic_score = if has_telematic { 0.9 } else { 0.1 };

    // Urgency score: Weighted combination
    let urgency_score = (
        warranty_score * 0.3_f32 +
        service_score * 0.3_f32 +
        stressor_score * 0.2_f32 +
        susceptibility_score * 0.1_f32 +
        telematic_score * 0.1_f32
    ).min(1.0_f32);

    // Calculate call_by_date: More urgent = sooner call date
    let days_until_call = if urgency_score > 0.8 {
        1 // Call tomorrow
    } else if urgency_score > 0.6 {
        3
    } else if urgency_score > 0.4 {
        7
    } else {
        14
    };
    let call_by_date = today + chrono::Duration::days(days_until_call);

    // Generate why_now message
    let why_now = generate_why_now(&vehicle, warranty_score, service_score, stressor_type.as_deref());
    
    // Generate suggested script
    let suggested_script = generate_script(&vehicle, stressor_type.as_deref(), warranty_score, service_score);

    LeadScores {
        urgency_score,
        stressor_score,
        warranty_score,
        susceptibility_score,
        telematic_score,
        has_telematic,
        stressor_type,
        why_now,
        call_by_date,
        suggested_script,
    }
}

fn generate_why_now(vehicle: &Vehicle, warranty_score: f32, service_score: f32, _stressor_type: Option<&str>) -> String {
    let mut reasons = Vec::new();
    
    if warranty_score > 0.7 {
        if let Some(exp_date) = vehicle.warranty_exp_date {
            let days = (exp_date - Utc::now().date_naive()).num_days();
            if days > 0 {
                reasons.push(format!("Warranty expires in {} days", days));
            } else {
                reasons.push("Warranty has expired".to_string());
            }
        }
    }
    
    if service_score > 0.7 {
        if let Some(last_service) = vehicle.last_service_date {
            let days = (Utc::now().date_naive() - last_service).num_days();
            reasons.push(format!("Last service was {} days ago", days));
        } else {
            reasons.push("No service record found".to_string());
        }
    }
    
    if reasons.is_empty() {
        reasons.push("Routine maintenance reminder".to_string());
    }
    
    format!("Customer should be contacted because: {}. This is an optimal time to reach out and provide value.", reasons.join(", "))
}

fn generate_script(vehicle: &Vehicle, stressor_type: Option<&str>, _warranty_score: f32, _service_score: f32) -> String {
    let customer_name = &vehicle.customer_name;
    let stressor = stressor_type.unwrap_or("maintenance");
    
    format!(
        "Hi {}, this is [Your Name] from [Dealership]. I wanted to reach out because your vehicle's {} is coming up. \
        We'd love to help ensure your vehicle stays in great condition. Would you be available for a quick conversation \
        about scheduling a service appointment? We can work around your schedule and make sure everything is taken care of.",
        customer_name, stressor
    )
}

