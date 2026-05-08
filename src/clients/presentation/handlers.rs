use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::clients::application::service::{ClientPortalService, ClientService, ClientSessionTypeService};
use crate::clients::domain::entity::{
    ApproveRescheduleInput, CreateClientInput, CreateClientSessionTypeInput,
    PortalRescheduleInput, UpdateClientInput, UpdateClientSessionTypeInput,
    UpdatePortalProfileInput, UpdateStatusInput,
};
use crate::scheduling::application::service::SessionService;
use crate::shared::config::AppConfig;
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

/// PUT /api/v1/portal/profiles/{client_id}
pub async fn portal_update_profile(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
    body: web::Json<UpdatePortalProfileInput>,
) -> Result<HttpResponse, AppError> {
    let profile = svc.update_profile(*client_id, user.id, &body).await?;
    Ok(HttpResponse::Ok().json(profile))
}

/// GET /api/v1/portal/profiles/{client_id}/invoices
pub async fn portal_list_invoices(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let invoices = svc.list_invoices(user.id, *client_id).await?;
    Ok(HttpResponse::Ok().json(invoices))
}

/// POST /api/v1/portal/sessions/{session_id}/reschedule
pub async fn portal_request_reschedule(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    path: web::Path<Uuid>,
    body: web::Json<PortalRescheduleInputBody>,
) -> Result<HttpResponse, AppError> {
    // Determine which client record belongs to this user for the session
    // We need to find the client_id from the session — do it via profiles then verify
    let profiles = svc.list_profiles(user.id).await?;
    if profiles.is_empty() {
        return Err(AppError::not_found("Client profile not found"));
    }

    let session_id = *path;

    // Try each profile until we find the one that owns this session
    let mut found = false;
    for profile in &profiles {
        let maybe_session = svc
            .portal_repo
            .find_session_by_id_for_client(session_id, profile.id)
            .await
            .map_err(AppError::from)?;
        if maybe_session.is_some() {
            // We know the client_id — request the reschedule
            let input = PortalRescheduleInput {
                requested_date: body.requested_date.clone(),
                requested_time: body.requested_time.clone(),
                reason: body.reason.clone(),
            };
            svc.request_reschedule(user.id, profile.id, session_id, &input).await?;
            found = true;
            break;
        }
    }

    if !found {
        return Err(AppError::not_found("Session not found or not accessible"));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct PortalRescheduleInputBody {
    pub requested_date: String,
    pub requested_time: String,
    pub reason: Option<String>,
}

/// GET /api/v1/portal/profiles/{client_id}/intake-forms
pub async fn portal_list_intake_forms(
    user: AuthUser,
    svc: web::Data<ClientPortalService>,
    client_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let forms = svc.list_intake_forms(user.id, *client_id).await?;
    Ok(HttpResponse::Ok().json(forms))
}

// ─── Therapist-side Reschedule Approve/Reject ─────────────────────────────

/// POST /api/v1/sessions/{id}/approve-reschedule
pub async fn approve_reschedule(
    user: AuthUser,
    id: web::Path<Uuid>,
    body: web::Json<ApproveRescheduleInput>,
    session_svc: web::Data<SessionService>,
    portal_svc: web::Data<ClientPortalService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc.get_by_id(user.id, *id).await?;
    if session.status != "reschedule_requested" {
        return Err(AppError::bad_request(
            "Session is not in reschedule_requested status",
        ));
    }

    if body.starts_at >= body.ends_at {
        return Err(AppError::bad_request("starts_at must be before ends_at"));
    }

    // Reschedule the session (clears reschedule fields, sets to scheduled)
    let updated_session = session_svc
        .reschedule(user.id, *id, body.starts_at, body.ends_at)
        .await?;

    // Clear reschedule request fields
    portal_svc
        .portal_repo
        .clear_reschedule_request(*id)
        .await
        .map_err(AppError::from)?;

    // Best-effort email to client
    let client_email = portal_svc
        .portal_repo
        .get_client_email_by_id(session.client_id)
        .await
        .unwrap_or(None);

    if let Some(ref email) = client_email {
        let ist_time = body.starts_at.with_timezone(&chrono_tz::Asia::Kolkata);
        let formatted = ist_time.format("%d %b %Y at %I:%M %p IST").to_string();
        let body_text = format!(
            "Good news! Your reschedule request has been approved.\n\nYour session is now confirmed for {}.\n\nSee you then!",
            formatted
        );
        let _ = crate::shared::email::send_email(
            &portal_svc.resend_api_key,
            crate::shared::email::EmailParams {
                to: email,
                reply_to: None,
                from_name: "Bendre",
                subject: "Your reschedule request has been approved",
                html: &body_text,
            },
        )
        .await;
    }

    Ok(HttpResponse::Ok().json(updated_session))
}

/// POST /api/v1/sessions/{id}/reject-reschedule
#[derive(Debug, Deserialize)]
pub struct RejectRescheduleInput {
    pub reason: Option<String>,
}

pub async fn reject_reschedule(
    user: AuthUser,
    id: web::Path<Uuid>,
    body: web::Json<RejectRescheduleInput>,
    session_svc: web::Data<SessionService>,
    portal_svc: web::Data<ClientPortalService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc.get_by_id(user.id, *id).await?;
    if session.status != "reschedule_requested" {
        return Err(AppError::bad_request(
            "Session is not in reschedule_requested status",
        ));
    }

    // Revert back to scheduled status
    session_svc
        .session_repo
        .update_status(user.id, *id, "scheduled", None, None, None)
        .await
        .map_err(AppError::from)?;

    // Clear reschedule request fields
    portal_svc
        .portal_repo
        .clear_reschedule_request(*id)
        .await
        .map_err(AppError::from)?;

    // Best-effort email to client
    let client_email = portal_svc
        .portal_repo
        .get_client_email_by_id(session.client_id)
        .await
        .unwrap_or(None);

    if let Some(ref email) = client_email {
        let ist_time = session.starts_at.with_timezone(&chrono_tz::Asia::Kolkata);
        let formatted = ist_time.format("%d %b %Y at %I:%M %p IST").to_string();
        let reason_note = body
            .reason
            .as_deref()
            .map(|r| format!("\n\nReason: {}", r))
            .unwrap_or_default();
        let body_text = format!(
            "Unfortunately, your reschedule request could not be accommodated.{}\n\nYour original session on {} is still confirmed. Please contact your therapist if you have any questions.",
            reason_note,
            formatted
        );
        let _ = crate::shared::email::send_email(
            &portal_svc.resend_api_key,
            crate::shared::email::EmailParams {
                to: email,
                reply_to: None,
                from_name: "Bendre",
                subject: "Your reschedule request was not approved",
                html: &body_text,
            },
        )
        .await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

// ─── Internal Endpoints ────────────────────────────────────────────────────

/// POST /api/v1/internal/send-session-reminders
/// Called by a cron/scheduler. Uses INTERNAL_API_KEY header for auth.
pub async fn send_session_reminders(
    req: actix_web::HttpRequest,
    portal_svc: web::Data<ClientPortalService>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    // Verify internal API key
    let api_key = std::env::var("INTERNAL_API_KEY").unwrap_or_default();
    if !api_key.is_empty() {
        let provided = req
            .headers()
            .get("X-Internal-Key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        if provided != api_key {
            return Err(AppError::unauthorized("Invalid internal API key"));
        }
    }

    let _ = config; // referenced to avoid unused warning

    let now = Utc::now();
    let from = now + chrono::Duration::hours(24);
    let to = now + chrono::Duration::hours(26);

    let sent = portal_svc
        .send_session_reminders(from, to)
        .await
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "reminders_sent": sent,
    })))
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
            // Client portal — profiles
            .route("/api/v1/portal/profiles", web::get().to(portal_list_profiles))
            .route("/api/v1/portal/profiles/{client_id}", web::get().to(portal_get_profile))
            .route("/api/v1/portal/profiles/{client_id}", web::put().to(portal_update_profile))
            // Client portal — sessions
            .route("/api/v1/portal/profiles/{client_id}/sessions/upcoming", web::get().to(portal_upcoming_sessions))
            .route("/api/v1/portal/profiles/{client_id}/sessions/upcoming/count", web::get().to(portal_upcoming_count))
            .route("/api/v1/portal/profiles/{client_id}/sessions/past", web::get().to(portal_past_sessions))
            // Client portal — invoices (with ownership verification)
            .route("/api/v1/portal/profiles/{client_id}/invoices", web::get().to(portal_list_invoices))
            // Client portal — intake forms
            .route("/api/v1/portal/profiles/{client_id}/intake-forms", web::get().to(portal_list_intake_forms))
            // Client portal — session reschedule request
            .route("/api/v1/portal/sessions/{session_id}/reschedule", web::post().to(portal_request_reschedule))
            // Therapist-side reschedule approval/rejection
            .route("/api/v1/sessions/{id}/approve-reschedule", web::post().to(approve_reschedule))
            .route("/api/v1/sessions/{id}/reject-reschedule", web::post().to(reject_reschedule))
            // Internal endpoints
            .route("/api/v1/internal/send-session-reminders", web::post().to(send_session_reminders))
            ;
}
