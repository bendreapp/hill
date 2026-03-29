mod shared;
mod system;

mod iam;
mod scheduling;
mod clients;
mod clinical;
mod billing;
mod engagement;
mod admin;
mod analytics;
mod leads;

use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::shared::auth::JwtKeys;
use crate::shared::config::AppConfig;
use crate::system::composition::AppServices;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    // Load config
    let config = AppConfig::from_env().expect("Missing required environment variables");
    tracing::info!("Starting Bendre API on {}:{}", config.host, config.port);

    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Database connected");

    // Initialize JWT keys (fetches JWKS from Supabase)
    let jwt_keys = JwtKeys::from_supabase(&config.supabase_url, &config.supabase_jwt_secret)
        .await
        .expect("Failed to initialize JWT keys");
    let jwt_keys = web::Data::new(Arc::new(jwt_keys));
    tracing::info!("JWT keys loaded");

    // Wire all services (composition root)
    let services = AppServices::new(pool.clone(), &config);

    // Wrap services in web::Data for Actix injection
    let therapist_svc = web::Data::new(services.therapist_service);
    let practice_svc = web::Data::new(services.practice_service);
    let onboarding_svc = web::Data::new(services.onboarding_service);
    let session_svc = web::Data::new(services.session_service);
    let booking_svc = web::Data::new(services.booking_service);
    let blocked_slot_svc = web::Data::new(services.blocked_slot_service);
    let recurring_svc = web::Data::new(services.recurring_service);
    let session_type_svc = web::Data::new(services.session_type_service);
    let client_svc = web::Data::new(services.client_service);
    let client_portal_svc = web::Data::new(services.client_portal_service);
    let note_svc = web::Data::new(services.note_service);
    let treatment_plan_svc = web::Data::new(services.treatment_plan_service);
    let message_svc = web::Data::new(services.message_service);
    let payment_svc = web::Data::new(services.payment_service);
    let resource_svc = web::Data::new(services.resource_service);
    let intake_svc = web::Data::new(services.intake_service);
    let broadcast_svc = web::Data::new(services.broadcast_service);
    let analytics_svc = web::Data::new(services.analytics_service);
    let lead_svc = web::Data::new(services.lead_service);
    let client_invitation_svc = web::Data::new(services.client_invitation_service);

    let db_pool = web::Data::new(pool.clone());
    let app_config = web::Data::new(config.clone());
    let frontend_url = config.frontend_url.clone();
    let hq_frontend_url = config.hq_frontend_url.clone();
    let host = config.host.clone();
    let port = config.port;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_origin(&hq_frontend_url)
            .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Authorization", "Content-Type", "Accept"])
            .max_age(3600);

        App::new()
            // Middleware
            .wrap(TracingLogger::default())
            .wrap(cors)
            .wrap(middleware::Compress::default())
            // Config & Auth
            .app_data(app_config.clone())
            .app_data(jwt_keys.clone())
            // IAM services
            .app_data(therapist_svc.clone())
            .app_data(practice_svc.clone())
            .app_data(onboarding_svc.clone())
            // Scheduling services
            .app_data(session_svc.clone())
            .app_data(booking_svc.clone())
            .app_data(blocked_slot_svc.clone())
            .app_data(recurring_svc.clone())
            .app_data(session_type_svc.clone())
            // Client services
            .app_data(client_svc.clone())
            .app_data(client_portal_svc.clone())
            // Clinical services
            .app_data(note_svc.clone())
            .app_data(treatment_plan_svc.clone())
            .app_data(message_svc.clone())
            // Billing services
            .app_data(payment_svc.clone())
            // Engagement services
            .app_data(resource_svc.clone())
            .app_data(intake_svc.clone())
            .app_data(broadcast_svc.clone())
            // Analytics services
            .app_data(analytics_svc.clone())
            // Leads services
            .app_data(lead_svc.clone())
            .app_data(client_invitation_svc.clone())
            // Database pool (for standalone endpoints)
            .app_data(db_pool.clone())
            // Health check & public endpoints
            .route("/health", web::get().to(health_check))
            .route("/api/v1/waitlist", web::post().to(join_waitlist))
            // Feature routes
            .configure(crate::iam::presentation::handlers::configure)
            .configure(crate::scheduling::presentation::handlers::configure)
            .configure(crate::clients::presentation::handlers::configure)
            .configure(crate::clinical::presentation::handlers::configure)
            .configure(crate::billing::presentation::handlers::configure)
            .configure(crate::engagement::presentation::handlers::configure)
            .configure(crate::analytics::presentation::handlers::configure)
            // Leads & invitations
            .configure(crate::leads::presentation::handlers::configure)
            // Admin routes
            .configure(crate::admin::handlers::configure)
    })
    .bind(format!("{}:{}", host, port))?
    .shutdown_timeout(30)
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "bendre-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[derive(Debug, Deserialize)]
struct WaitlistInput {
    email: String,
    source: Option<String>,
}

async fn join_waitlist(
    pool: web::Data<PgPool>,
    input: web::Json<WaitlistInput>,
) -> HttpResponse {
    let email = input.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid email address"
        }));
    }

    let result = sqlx::query_scalar::<_, String>(
        "INSERT INTO waitlist (email, source) VALUES ($1, $2)
         ON CONFLICT (email) DO NOTHING
         RETURNING email"
    )
    .bind(&email)
    .bind(input.source.as_deref().unwrap_or("website"))
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Created().json(serde_json::json!({
            "message": "You're on the list!"
        })),
        Ok(None) => HttpResponse::Ok().json(serde_json::json!({
            "message": "You're already on the list!"
        })),
        Err(e) => {
            tracing::error!("Waitlist insert error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Something went wrong. Please try again."
            }))
        }
    }
}
