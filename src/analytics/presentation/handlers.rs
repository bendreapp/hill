use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::analytics::application::service::AnalyticsService;
use crate::shared::error::AppError;
use crate::shared::types::AuthUser;

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct TopClientsQuery {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub limit: Option<i64>,
}

pub async fn overview(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let stats = svc.overview(&[user.id], query.start, query.end).await?;
    Ok(HttpResponse::Ok().json(stats))
}

pub async fn revenue_by_month(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let data = svc
        .revenue_by_month(&[user.id], query.start, query.end)
        .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn sessions_by_month(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let data = svc
        .sessions_by_month(&[user.id], query.start, query.end)
        .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn client_growth(
    user: AuthUser,
    svc: web::Data<AnalyticsService>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    let data = svc
        .client_growth(&[user.id], query.start, query.end)
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
        .top_clients(&[user.id], query.start, query.end, limit)
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
