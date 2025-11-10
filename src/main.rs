use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use stressor_leads::{
    db::{create_pool, run_migrations},
    handlers::*,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Create database pool
    let pool = create_pool().await?;
    println!("Connected to database successfully!");

    // Run migrations
    run_migrations(&pool).await?;
    println!("Migrations completed successfully!");

    // Build CORS layer - allow frontend domains
    // Note: When allow_credentials(true), cannot use allow_origin(Any) or allow_headers(Any)
    // Must specify both origin and headers explicitly
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "https://stressor-leads-frontend-1jwz0mzvn-sams-projects-bf92499c.vercel.app".to_string());
    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse().unwrap()) // Use specific origin from FRONTEND_URL
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ACCEPT_LANGUAGE,
        ])
        .allow_credentials(true);

    // Build application with routes
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/dealers/register", post(register_dealer))
        .route("/api/dealers/login", post(login_dealer))
        .route("/api/dealers/me", get(get_dealer_profile))
        .route("/api/uploads", post(upload_file))
        .route("/api/uploads", get(list_uploads))
        .route("/api/uploads/:id", get(get_upload))
        .route("/api/vehicles", get(list_vehicles))
        .route("/api/vehicles/:id", get(get_vehicle))
        .route("/api/scored-leads", get(list_scored_leads))
        .route("/api/scored-leads/:id", get(get_scored_lead))
        .layer(cors)
        .with_state(pool);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
