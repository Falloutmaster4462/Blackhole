mod api;
mod db;
mod models;
mod realtime;
mod rules;
mod smtp;

use anyhow::Result;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blackhole_mail=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/blackhole".to_string());
    
    tracing::info!("Starting Blackhole Mail...");
    
    // Create database pool
    let pool = db::create_pool(&database_url).await?;
    tracing::info!("Database pool created");
    
    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");
    
    // Create rules engine
    let rules_engine = Arc::new(rules::RulesEngine::new(pool.clone()));
    tracing::info!("Rules engine initialized");
    
    // Create real-time state
    let realtime_state = Arc::new(realtime::RealtimeState::new());
    tracing::info!("Real-time event system initialized");
    
    // Start SMTP server
    let smtp_server = Arc::new(smtp::SmtpServer::new(
        pool.clone(),
        Arc::clone(&rules_engine),
        2525,
    ));
    
    let smtp_handle = tokio::spawn(async move {
        if let Err(e) = smtp_server.start().await {
            tracing::error!("SMTP server error: {:?}", e);
        }
    });
    
    tracing::info!("SMTP server started on port 2525");
    
    // Create API state
    let api_state = api::AppState {
        pool: pool.clone(),
        rules_engine: Arc::clone(&rules_engine),
    };
    
    // Build API router
    let api_router = api::create_router(api_state);
    
    // Build SSE router
    let sse_router = realtime::create_sse_router(Arc::clone(&realtime_state));
    
    // Combine routers
    let app = api_router
        .merge(sse_router)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE]),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http());
    
    // Start HTTP server
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP server listening on {}", addr);
    
    tracing::info!("🕳️  Blackhole Mail is ready!");
    tracing::info!("  - API: http://localhost:8080");
    tracing::info!("  - SMTP: localhost:2525");
    tracing::info!("  - Frontend: http://localhost:3000 (run separately)");
    
    axum::serve(listener, app).await?;
    
    // Wait for SMTP server to finish (it runs indefinitely)
    let _ = smtp_handle.await;
    
    Ok(())
}
