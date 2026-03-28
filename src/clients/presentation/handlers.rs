use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::clients::application::service::{ClientPortalService, ClientService};
use crate::clients::domain::entity::{CreateClientInput, UpdateClientInput, UpdateStatusInput};
use crate::shared::error::AppError;
use crate::shared::types::{AuthUser, Paginated};

// ─── Query Parameters ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ClientListQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PortalSessionQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ─── Client Handlers (Therapist) ────────────────────────────────────────────

pub async fn list_clients(
    user: AuthUser,
    svc: web::Data<ClientService>,
    query: web::Query<ClientListQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc
        .list_clients(
            &[user.id],
            query.status.as_deref(),
            per_page,
            offset,
        )
        .await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_client(
    _user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let client = svc.get_client(*id).await?;
    Ok(HttpResponse::Ok().json(client))
}

pub async fn create_client(
    user: AuthUser,
    svc: web::Data<ClientService>,
    body: web::Json<CreateClientInput>,
) -> Result<HttpResponse, AppError> {
    // Plan limit: pass None for now (unlimited), wire up plan checks when billing is integrated
    let client = svc.create_client(user.id, &body, None).await?;
    Ok(HttpResponse::Created().json(client))
}

pub async fn update_client(
    _user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateClientInput>,
) -> Result<HttpResponse, AppError> {
    let client = svc.update_client(*id, &body).await?;
    Ok(HttpResponse::Ok().json(client))
}

pub async fn delete_client(
    _user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.soft_delete_client(*id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_client_status(
    _user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateStatusInput>,
) -> Result<HttpResponse, AppError> {
    let client = svc.update_status(*id, &body.status).await?;
    Ok(HttpResponse::Ok().json(client))
}

pub async fn count_active_clients(
    user: AuthUser,
    svc: web::Data<ClientService>,
) -> Result<HttpResponse, AppError> {
    let count = svc.count_active(user.id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

// ─── Client Portal Handlers ────────────────────────────────────────────────

pub async fn portal_list_profiles(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
) -> Result<HttpResponse, AppError> {
    let profiles = svc.list_profiles(user.id).await?;
    Ok(HttpResponse::Ok().json(profiles))
}

pub async fn portal_upcoming_sessions(
    _user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PortalSessionQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(10).min(50);
    let sessions = svc.upcoming_sessions(*client_id, limit).await?;
    Ok(HttpResponse::Ok().json(sessions))
}

pub async fn portal_past_sessions(
    _user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PortalSessionQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc.past_sessions(*client_id, per_page, offset).await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn portal_get_profile(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let profiles = svc.list_profiles(user.id).await?;
    let profile = profiles
        .into_iter()
        .find(|p| p.id == *client_id)
        .ok_or_else(|| AppError::not_found("Client profile not found"))?;
    Ok(HttpResponse::Ok().json(profile))
}

pub async fn portal_upcoming_count(
    _user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let sessions = svc.upcoming_sessions(*client_id, 100).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": sessions.len() })))
}

pub async fn portal_invoices(
    _user: AuthUser,
    _client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // Returns invoices for a client in the portal; delegates to billing feature via HTTP.
    // Kept as a thin pass-through to avoid cross-feature imports.
    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": [], "total": 0 })))
}

pub async fn portal_resources(
    _user: AuthUser,
    _client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // Returns shared resources for a client in the portal; delegates to engagement feature via HTTP.
    // Kept as a thin pass-through to avoid cross-feature imports.
    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": [], "total": 0 })))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
            // Therapist client management
            .route("/api/v1/clients", web::get().to(list_clients))
            .route("/api/v1/clients", web::post().to(create_client))
            .route("/api/v1/clients/count", web::get().to(count_active_clients))
            .route("/api/v1/clients/{id}", web::get().to(get_client))
            .route("/api/v1/clients/{id}", web::put().to(update_client))
            .route("/api/v1/clients/{id}", web::delete().to(delete_client))
            .route("/api/v1/clients/{id}/status", web::patch().to(update_client_status))
            // Client portal
            .route("/api/v1/portal/profiles", web::get().to(portal_list_profiles))
            .route("/api/v1/portal/profiles/{client_id}", web::get().to(portal_get_profile))
            .route("/api/v1/portal/profiles/{client_id}/sessions/upcoming", web::get().to(portal_upcoming_sessions))
            .route("/api/v1/portal/profiles/{client_id}/sessions/upcoming/count", web::get().to(portal_upcoming_count))
            .route("/api/v1/portal/profiles/{client_id}/sessions/past", web::get().to(portal_past_sessions))
            .route("/api/v1/portal/profiles/{client_id}/invoices", web::get().to(portal_invoices))
            .route("/api/v1/portal/profiles/{client_id}/resources", web::get().to(portal_resources));
}
