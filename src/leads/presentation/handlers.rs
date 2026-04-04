use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::iam::application::service::TherapistService;
use crate::leads::application::service::{ClientInvitationService, LeadService};
use crate::leads::domain::entity::{CreateLeadInput, UpdateLeadInput};
use crate::shared::error::AppError;
use crate::shared::types::AuthUser;

// ─── Query Parameters ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LeadListQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct InviteClientInput {
    pub client_id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
}

// ─── Lead Handlers ─────────────────────────────────────────────────────────

pub async fn list_leads(
    user: AuthUser,
    svc: web::Data<LeadService>,
    query: web::Query<LeadListQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);
    let (leads, total) = svc
        .list_leads(user.id, query.status.as_deref(), limit, offset)
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": leads,
        "total": total,
    })))
}

pub async fn get_lead(
    user: AuthUser,
    id: web::Path<Uuid>,
    svc: web::Data<LeadService>,
) -> Result<HttpResponse, AppError> {
    let lead = svc.get_lead(id.into_inner(), user.id).await?;
    Ok(HttpResponse::Ok().json(lead))
}

pub async fn create_lead(
    user: AuthUser,
    svc: web::Data<LeadService>,
    body: web::Json<CreateLeadInput>,
) -> Result<HttpResponse, AppError> {
    let lead = svc.create_lead(user.id, &body).await?;
    Ok(HttpResponse::Created().json(lead))
}

pub async fn update_lead(
    user: AuthUser,
    id: web::Path<Uuid>,
    svc: web::Data<LeadService>,
    body: web::Json<UpdateLeadInput>,
) -> Result<HttpResponse, AppError> {
    let lead = svc.update_lead(id.into_inner(), user.id, &body).await?;
    Ok(HttpResponse::Ok().json(lead))
}

// ─── Public Booking Endpoints ───────────────────────────────────────────────

/// Public — GET /api/v1/booking/{slug}/profile
/// Returns therapist public profile info (no auth)
pub async fn get_public_profile(
    slug: web::Path<String>,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_by_slug(&slug).await?;
    let mut profile = serde_json::json!({
        "slug": therapist.slug,
        "full_name": therapist.full_name,
        "display_name": therapist.display_name,
        "bio": therapist.bio,
        "avatar_url": therapist.avatar_url,
        "show_pricing": therapist.show_pricing,
    });
    if therapist.show_pricing {
        profile["session_rate_inr"] = serde_json::json!(therapist.session_rate_inr);
    }
    Ok(HttpResponse::Ok().json(profile))
}

/// Public — POST /api/v1/booking/{slug}/inquire
/// Creates a lead from the public booking page (no auth)
pub async fn public_inquire(
    slug: web::Path<String>,
    lead_svc: web::Data<LeadService>,
    body: web::Json<CreateLeadInput>,
) -> Result<HttpResponse, AppError> {
    let lead = lead_svc.create_lead_by_slug(&slug, &body).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "id": lead.id,
    })))
}

// ─── Client Invitation Handlers ────────────────────────────────────────────

pub async fn invite_client(
    user: AuthUser,
    svc: web::Data<ClientInvitationService>,
    body: web::Json<InviteClientInput>,
) -> Result<HttpResponse, AppError> {
    let invitation = svc
        .create_invitation(
            user.id,
            body.client_id,
            body.email.as_deref(),
            body.phone.as_deref(),
        )
        .await?;
    Ok(HttpResponse::Created().json(invitation))
}

/// Public — fetch invitation by token (no auth needed)
pub async fn get_invitation_by_token(
    token: web::Path<String>,
    svc: web::Data<ClientInvitationService>,
) -> Result<HttpResponse, AppError> {
    let invitation = svc.get_by_token(&token).await?;
    Ok(HttpResponse::Ok().json(invitation))
}

/// Public — claim invitation (client sets up account)
pub async fn claim_invitation(
    token: web::Path<String>,
    svc: web::Data<ClientInvitationService>,
) -> Result<HttpResponse, AppError> {
    let invitation = svc.claim(&token).await?;
    Ok(HttpResponse::Ok().json(invitation))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Leads (authenticated)
        .route("/api/v1/leads", web::get().to(list_leads))
        .route("/api/v1/leads", web::post().to(create_lead))
        .route("/api/v1/leads/{id}", web::get().to(get_lead))
        .route("/api/v1/leads/{id}", web::put().to(update_lead))
        // Public booking endpoints (no auth)
        .route("/api/v1/booking/{slug}/profile", web::get().to(get_public_profile))
        .route("/api/v1/booking/{slug}/inquire", web::post().to(public_inquire))
        // Client invitations (authenticated)
        .route("/api/v1/client-invitations", web::post().to(invite_client))
        // Client invitations (public — for claiming)
        .route("/api/v1/client-invitations/by-token/{token}", web::get().to(get_invitation_by_token))
        .route("/api/v1/client-invitations/by-token/{token}/claim", web::post().to(claim_invitation));
}
