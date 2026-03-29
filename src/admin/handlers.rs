use actix_web::{dev::Payload, web, FromRequest, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::auth::JwtKeys;
use crate::shared::config::AppConfig;
use crate::shared::error::AppError;

// ─── Admin Auth Extractor ───────────────────────────────────────────────────

pub struct AdminUser {
    pub id: Uuid,
    pub email: String,
}

impl FromRequest for AdminUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let jwt_keys = req
            .app_data::<web::Data<Arc<JwtKeys>>>()
            .cloned();

        let config = req
            .app_data::<web::Data<AppConfig>>()
            .cloned();

        Box::pin(async move {
            let header = auth_header
                .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

            let token = header
                .strip_prefix("Bearer ")
                .ok_or_else(|| AppError::unauthorized("Invalid Authorization header format"))?;

            let keys = jwt_keys
                .ok_or_else(|| AppError::internal("JWT keys not configured"))?;

            let user = keys.verify(token)?;

            let config = config
                .ok_or_else(|| AppError::internal("Config not available"))?;

            if !config.admin_emails.contains(&user.email.to_lowercase()) {
                return Err(AppError::forbidden("Not an admin"));
            }

            Ok(AdminUser {
                id: user.id,
                email: user.email,
            })
        })
    }
}

// ─── Response Types ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PlatformStats {
    total_therapists: i64,
    total_clients: i64,
    total_sessions: i64,
    completed_sessions: i64,
    waitlist_count: i64,
    new_therapists_30d: i64,
    new_clients_30d: i64,
}

#[derive(Serialize, sqlx::FromRow)]
struct TherapistRow {
    id: Uuid,
    full_name: String,
    display_name: Option<String>,
    phone: Option<String>,
    slug: String,
    timezone: String,
    booking_page_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
struct WaitlistRow {
    id: Uuid,
    email: String,
    source: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    therapist_id: Uuid,
    client_id: Uuid,
    status: String,
    starts_at: chrono::DateTime<chrono::Utc>,
    ends_at: chrono::DateTime<chrono::Utc>,
    duration_mins: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
struct SignupDay {
    date: chrono::NaiveDate,
    count: i64,
}

#[derive(Serialize)]
struct TherapistDetail {
    #[serde(flatten)]
    therapist: TherapistDetailRow,
    client_count: i64,
    session_count: i64,
    completed_count: i64,
}

#[derive(Serialize, sqlx::FromRow)]
struct TherapistDetailRow {
    id: Uuid,
    full_name: String,
    display_name: Option<String>,
    phone: Option<String>,
    slug: String,
    timezone: String,
    qualifications: Option<String>,
    booking_page_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct DaysQuery {
    pub days: Option<i32>,
}

// ─── Handlers ───────────────────────────────────────────────────────────────

pub async fn platform_stats(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64)>(
        "SELECT
            (SELECT COUNT(*) FROM therapists)::bigint,
            (SELECT COUNT(*) FROM clients WHERE deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM sessions WHERE deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM sessions WHERE status = 'completed' AND deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM waitlist)::bigint,
            (SELECT COUNT(*) FROM therapists WHERE created_at >= NOW() - INTERVAL '30 days')::bigint,
            (SELECT COUNT(*) FROM clients WHERE created_at >= NOW() - INTERVAL '30 days' AND deleted_at IS NULL)::bigint"
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin stats query error: {}", e);
        AppError::internal("Failed to fetch stats")
    })?;

    Ok(HttpResponse::Ok().json(PlatformStats {
        total_therapists: stats.0,
        total_clients: stats.1,
        total_sessions: stats.2,
        completed_sessions: stats.3,
        waitlist_count: stats.4,
        new_therapists_30d: stats.5,
        new_clients_30d: stats.6,
    }))
}

pub async fn list_therapists(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query_as::<_, TherapistRow>(
        "SELECT id, full_name, display_name, phone, slug, timezone, booking_page_active, created_at, updated_at
         FROM therapists ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin therapists query error: {}", e);
        AppError::internal("Failed to fetch therapists")
    })?;

    Ok(HttpResponse::Ok().json(rows))
}

pub async fn therapist_detail(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let therapist = sqlx::query_as::<_, TherapistDetailRow>(
        "SELECT id, full_name, display_name, phone, slug, timezone, qualifications, booking_page_active, created_at, updated_at
         FROM therapists WHERE id = $1"
    )
    .bind(id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin therapist detail error: {}", e);
        AppError::internal("Failed to fetch therapist")
    })?
    .ok_or_else(|| AppError::not_found("Therapist"))?;

    let counts = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT
            (SELECT COUNT(*) FROM clients WHERE therapist_id = $1 AND deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM sessions WHERE therapist_id = $1 AND deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM sessions WHERE therapist_id = $1 AND status = 'completed')::bigint"
    )
    .bind(therapist.id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin therapist counts error: {}", e);
        AppError::internal("Failed to fetch counts")
    })?;

    Ok(HttpResponse::Ok().json(TherapistDetail {
        therapist,
        client_count: counts.0,
        session_count: counts.1,
        completed_count: counts.2,
    }))
}

pub async fn list_waitlist(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query_as::<_, WaitlistRow>(
        "SELECT id, email, source, created_at FROM waitlist ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin waitlist query error: {}", e);
        AppError::internal("Failed to fetch waitlist")
    })?;

    Ok(HttpResponse::Ok().json(rows))
}

pub async fn recent_sessions(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<LimitQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(50).min(200);

    let rows = sqlx::query_as::<_, SessionRow>(
        "SELECT id, therapist_id, client_id, status::text as status, starts_at, ends_at, duration_mins, created_at
         FROM sessions WHERE deleted_at IS NULL
         ORDER BY starts_at DESC LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin sessions query error: {}", e);
        AppError::internal("Failed to fetch sessions")
    })?;

    Ok(HttpResponse::Ok().json(rows))
}

pub async fn signups_by_day(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<DaysQuery>,
) -> Result<HttpResponse, AppError> {
    let days = query.days.unwrap_or(30).min(365);

    let rows = sqlx::query_as::<_, SignupDay>(
        "SELECT created_at::date AS date, COUNT(*)::bigint AS count
         FROM therapists
         WHERE created_at >= NOW() - make_interval(days => $1)
         GROUP BY created_at::date
         ORDER BY date"
    )
    .bind(days)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin signups query error: {}", e);
        AppError::internal("Failed to fetch signups")
    })?;

    Ok(HttpResponse::Ok().json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
struct CategoryCount {
    category: String,
    count: i64,
}

#[derive(Serialize, sqlx::FromRow)]
struct MonthCount {
    month: String,
    count: i64,
}

#[derive(Serialize)]
struct ClientStats {
    total: i64,
    active: i64,
    inactive: i64,
    categories: Vec<CategoryCount>,
    growth_by_month: Vec<MonthCount>,
}

pub async fn client_stats(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let counts = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT
            (SELECT COUNT(*) FROM clients WHERE deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM clients WHERE is_active = true AND deleted_at IS NULL)::bigint,
            (SELECT COUNT(*) FROM clients WHERE is_active = false AND deleted_at IS NULL)::bigint"
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Admin client counts error: {}", e);
        AppError::internal("Failed to fetch client stats")
    })?;

    let categories = sqlx::query_as::<_, CategoryCount>(
        "SELECT COALESCE(category::text, 'uncategorized') AS category, COUNT(*)::bigint AS count
         FROM clients WHERE deleted_at IS NULL
         GROUP BY category ORDER BY count DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let growth = sqlx::query_as::<_, MonthCount>(
        "SELECT to_char(created_at, 'YYYY-MM') AS month, COUNT(*)::bigint AS count
         FROM clients WHERE deleted_at IS NULL AND created_at >= NOW() - INTERVAL '180 days'
         GROUP BY to_char(created_at, 'YYYY-MM')
         ORDER BY month"
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    Ok(HttpResponse::Ok().json(ClientStats {
        total: counts.0,
        active: counts.1,
        inactive: counts.2,
        categories,
        growth_by_month: growth,
    }))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/api/v1/admin/stats", web::get().to(platform_stats))
        .route("/api/v1/admin/therapists", web::get().to(list_therapists))
        .route("/api/v1/admin/therapists/{id}", web::get().to(therapist_detail))
        .route("/api/v1/admin/waitlist", web::get().to(list_waitlist))
        .route("/api/v1/admin/sessions/recent", web::get().to(recent_sessions))
        .route("/api/v1/admin/signups-by-day", web::get().to(signups_by_day))
        .route("/api/v1/admin/clients/stats", web::get().to(client_stats));
}
