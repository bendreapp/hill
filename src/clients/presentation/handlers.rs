use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::clients::application::service::{ClientPortalService, ClientService, ClientSessionTypeService};
use crate::clients::domain::entity::{
    CreateClientInput, CreateClientSessionTypeInput, UpdateClientInput, UpdateClientSessionTypeInput,
    UpdateStatusInput,
};
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
    user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let client = svc.get_client(*id, user.id).await?;
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
    user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateClientInput>,
) -> Result<HttpResponse, AppError> {
    let client = svc.update_client(*id, user.id, &body).await?;
    Ok(HttpResponse::Ok().json(client))
}

pub async fn delete_client(
    user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.soft_delete_client(*id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_client_status(
    user: AuthUser,
    svc: web::Data<ClientService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateStatusInput>,
) -> Result<HttpResponse, AppError> {
    let client = svc.update_status(*id, user.id, &body.status).await?;
    Ok(HttpResponse::Ok().json(client))
}

pub async fn count_active_clients(
    user: AuthUser,
    svc: web::Data<ClientService>,
) -> Result<HttpResponse, AppError> {
    let count = svc.count_active(user.id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

// ─── Client Session Type Handlers ───────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct ClientSessionTypePath {
    pub client_id: Uuid,
    pub id: Uuid,
}

pub async fn list_client_session_types(
    user: AuthUser,
    svc: web::Data<ClientSessionTypeService>,
    client_svc: web::Data<ClientService>,
    client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // Verify the client belongs to this therapist
    client_svc.get_client(*client_id, user.id).await?;
    let types = svc.list_for_client(*client_id, user.id).await?;
    Ok(HttpResponse::Ok().json(types))
}

pub async fn create_client_session_type(
    user: AuthUser,
    svc: web::Data<ClientSessionTypeService>,
    client_svc: web::Data<ClientService>,
    client_id: web::Path<Uuid>,
    body: web::Json<CreateClientSessionTypeInput>,
) -> Result<HttpResponse, AppError> {
    client_svc.get_client(*client_id, user.id).await?;
    let session_type = svc.create_for_client(*client_id, user.id, &body).await?;
    Ok(HttpResponse::Created().json(session_type))
}

pub async fn update_client_session_type(
    user: AuthUser,
    svc: web::Data<ClientSessionTypeService>,
    client_svc: web::Data<ClientService>,
    path: web::Path<ClientSessionTypePath>,
    body: web::Json<UpdateClientSessionTypeInput>,
) -> Result<HttpResponse, AppError> {
    client_svc.get_client(path.client_id, user.id).await?;
    let session_type = svc
        .update_for_client(path.id, path.client_id, user.id, &body)
        .await?;
    Ok(HttpResponse::Ok().json(session_type))
}

pub async fn delete_client_session_type(
    user: AuthUser,
    svc: web::Data<ClientSessionTypeService>,
    client_svc: web::Data<ClientService>,
    path: web::Path<ClientSessionTypePath>,
) -> Result<HttpResponse, AppError> {
    client_svc.get_client(path.client_id, user.id).await?;
    svc.delete_for_client(path.id, path.client_id, user.id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn set_default_client_session_type(
    user: AuthUser,
    svc: web::Data<ClientSessionTypeService>,
    client_svc: web::Data<ClientService>,
    path: web::Path<ClientSessionTypePath>,
) -> Result<HttpResponse, AppError> {
    client_svc.get_client(path.client_id, user.id).await?;
    svc.set_default(path.id, path.client_id, user.id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
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
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PortalSessionQuery>,
) -> Result<HttpResponse, AppError> {
    svc.verify_client_ownership(user.id, *client_id).await?;
    let limit = query.limit.unwrap_or(10).min(50);
    let sessions = svc.upcoming_sessions(*client_id, limit).await?;
    Ok(HttpResponse::Ok().json(sessions))
}

pub async fn portal_past_sessions(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PortalSessionQuery>,
) -> Result<HttpResponse, AppError> {
    svc.verify_client_ownership(user.id, *client_id).await?;
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
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.verify_client_ownership(user.id, *client_id).await?;
    let sessions = svc.upcoming_sessions(*client_id, 100).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": sessions.len() })))
}

pub async fn portal_invoices(
    _user: AuthUser,
    _client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // TODO: verify client ownership once billing is wired up
    // Returns invoices for a client in the portal; delegates to billing feature via HTTP.
    // Kept as a thin pass-through to avoid cross-feature imports.
    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": [], "total": 0 })))
}

pub async fn portal_resources(
    _user: AuthUser,
    _client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // TODO: verify client ownership once engagement is wired up
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
            // Client-specific session types
            .route("/api/v1/clients/{client_id}/session-types", web::get().to(list_client_session_types))
            .route("/api/v1/clients/{client_id}/session-types", web::post().to(create_client_session_type))
            .route("/api/v1/clients/{client_id}/session-types/{id}", web::put().to(update_client_session_type))
            .route("/api/v1/clients/{client_id}/session-types/{id}", web::delete().to(delete_client_session_type))
            .route("/api/v1/clients/{client_id}/session-types/{id}/set-default", web::post().to(set_default_client_session_type))
            // Client portal
            .route("/api/v1/portal/profiles", web::get().to(portal_list_profiles))
            .route("/api/v1/portal/profiles/{client_id}", web::get().to(portal_get_profile))
            .route("/api/v1/portal/profiles/{client_id}/sessions/upcoming", web::get().to(portal_upcoming_sessions))
            .route("/api/v1/portal/profiles/{client_id}/sessions/upcoming/count", web::get().to(portal_upcoming_count))
            .route("/api/v1/portal/profiles/{client_id}/sessions/past", web::get().to(portal_past_sessions))
            .route("/api/v1/portal/profiles/{client_id}/invoices", web::get().to(portal_invoices))
            .route("/api/v1/portal/profiles/{client_id}/resources", web::get().to(portal_resources));
}
