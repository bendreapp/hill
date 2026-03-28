use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::analytics::application::service::AnalyticsService;
use crate::shared::error::AppError;
use crate::shared::types::AuthUser;

fn parse_date(s: &str) -> Result<NaiveDate, AppError> {
    // Try ISO datetime first, extract date part
    if s.contains('T') {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Ok(dt.naive_utc().date());
        }
        if let Ok(dt) = s.parse::<chrono::DateTime<chrono::Utc>>() {
            return Ok(dt.naive_utc().date());
        }
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| AppError::bad_request("Invalid date format"))
}

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct TopClientsQuery {
    pub start: String,
    pub end: String,
    pub limit: Option<i64>,
}

pub async fn overview(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let stats = svc.overview(&[user.id], parse_date(&query.start)?, parse_date(&query.end)?).await?;
    Ok(HttpResponse::Ok().json(stats))
}

pub async fn revenue_by_month(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let data = svc
        .revenue_by_month(&[user.id], parse_date(&query.start)?, parse_date(&query.end)?)
        .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn sessions_by_month(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let data = svc
        .sessions_by_month(&[user.id], parse_date(&query.start)?, parse_date(&query.end)?)
        .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn client_growth(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let data = svc
        .client_growth(&[user.id], parse_date(&query.start)?, parse_date(&query.end)?)
        .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn top_clients(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<TopClientsQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(10).min(50);
    let data = svc
        .top_clients(&[user.id], parse_date(&query.start)?, parse_date(&query.end)?, limit)
        .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn client_category_breakdown(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
) -> Result<HttpResponse, AppError> {
    let data = svc.client_category_breakdown(&[user.id]).await?;
    Ok(HttpResponse::Ok().json(data))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
.route("/api/v1/analytics/overview", web::get().to(overview))
            .route("/api/v1/analytics/revenue", web::get().to(revenue_by_month))
            .route("/api/v1/analytics/sessions", web::get().to(sessions_by_month))
            .route("/api/v1/analytics/client-growth", web::get().to(client_growth))
            .route("/api/v1/analytics/top-clients", web::get().to(top_clients))
            .route("/api/v1/analytics/client-categories", web::get().to(client_category_breakdown));
}
