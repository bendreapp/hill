use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::iam::application::service::{TherapistService, PracticeService, OnboardingService};
use crate::shared::error::AppError;
use crate::shared::types::AuthUser;

// ─── Therapist Handlers ──────────────────────────────────────────────────────

pub async fn get_me(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_me(user.id).await?;
    Ok(HttpResponse::Ok().json(therapist))
}

pub async fn get_by_slug(
    slug: web::Path<String>,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_by_slug(&slug).await?;
    Ok(HttpResponse::Ok().json(therapist))
}

pub async fn get_availability(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let availability = therapist_svc.get_availability(user.id).await?;
    Ok(HttpResponse::Ok().json(availability))
}

// ─── Practice Handlers ───────────────────────────────────────────────────────

pub async fn get_my_practice(
    user: AuthUser,
    practice_svc: web::Data<PracticeService>,
) -> Result<HttpResponse, AppError> {
    let result = practice_svc.get_my_practice(user.id).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn list_members(
    user: AuthUser,
    practice_svc: web::Data<PracticeService>,
    practice_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // Verify the user is a member of this practice
    let membership = practice_svc.get_my_practice(user.id).await?;
    match membership {
        Some((practice, _)) if practice.id == *practice_id => {},
        _ => return Err(AppError::not_found("Practice not found")),
    }
    let members = practice_svc.list_members(*practice_id).await?;
    Ok(HttpResponse::Ok().json(members))
}

// ─── Onboarding Handlers ─────────────────────────────────────────────────────

pub async fn list_onboarding_tokens(
    user: AuthUser,
    onboarding_svc: web::Data<OnboardingService>,
) -> Result<HttpResponse, AppError> {
    let tokens = onboarding_svc.list_tokens(user.id).await?;
    Ok(HttpResponse::Ok().json(tokens))
}

pub async fn get_onboarding_token(
    token: web::Path<Uuid>,
    onboarding_svc: web::Data<OnboardingService>,
) -> Result<HttpResponse, AppError> {
    let t = onboarding_svc.validate_token(*token).await?;
    Ok(HttpResponse::Ok().json(t))
}

// ─── Therapist Update ────────────────────────────────────────────────────────

pub async fn update_me(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let mut therapist = therapist_svc.get_me(user.id).await?;

    // Apply partial updates from JSON body
    let updates = body.into_inner();
    if let Some(v) = updates.get("full_name").and_then(|v| v.as_str()) { therapist.full_name = v.to_string(); }
    if let Some(v) = updates.get("display_name") { therapist.display_name = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("bio") { therapist.bio = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("qualifications") { therapist.qualifications = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("phone") { therapist.phone = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("slug").and_then(|v| v.as_str()) { therapist.slug = v.to_string(); }
    if let Some(v) = updates.get("gstin") { therapist.gstin = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("booking_page_active").and_then(|v| v.as_bool()) { therapist.booking_page_active = v; }
    if let Some(v) = updates.get("show_pricing").and_then(|v| v.as_bool()) { therapist.show_pricing = v; }
    if let Some(v) = updates.get("session_duration_mins").and_then(|v| v.as_i64()) { therapist.session_duration_mins = v as i32; }
    if let Some(v) = updates.get("buffer_mins").and_then(|v| v.as_i64()) { therapist.buffer_mins = v as i32; }
    if let Some(v) = updates.get("session_rate_inr").and_then(|v| v.as_i64()) { therapist.session_rate_inr = v as i32; }
    if let Some(v) = updates.get("cancellation_hours").and_then(|v| v.as_i64()) { therapist.cancellation_hours = v as i32; }
    if let Some(v) = updates.get("min_booking_advance_hours").and_then(|v| v.as_i64()) { therapist.min_booking_advance_hours = v as i32; }
    if let Some(v) = updates.get("no_show_charge_percent").and_then(|v| v.as_i64()) { therapist.no_show_charge_percent = v as i32; }
    if let Some(v) = updates.get("late_cancel_charge_percent").and_then(|v| v.as_i64()) { therapist.late_cancel_charge_percent = v as i32; }
    if updates.get("custom_tags").is_some() { therapist.custom_tags = updates.get("custom_tags").cloned(); }
    if let Some(v) = updates.get("cancellation_policy") { therapist.cancellation_policy = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("late_policy") { therapist.late_policy = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("rescheduling_policy") { therapist.rescheduling_policy = v.as_str().map(|s| s.to_string()); }
    // Onboarding / plan fields
    if let Some(v) = updates.get("whatsapp_number") { therapist.whatsapp_number = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("team_size") { therapist.team_size = v.as_i64().map(|n| n as i32); }
    if let Some(v) = updates.get("comms_whatsapp").and_then(|v| v.as_bool()) { therapist.comms_whatsapp = v; }
    if let Some(v) = updates.get("comms_email").and_then(|v| v.as_bool()) { therapist.comms_email = v; }
    if let Some(v) = updates.get("comms_sms").and_then(|v| v.as_bool()) { therapist.comms_sms = v; }
    if let Some(v) = updates.get("avatar_key") { therapist.avatar_key = v.as_str().map(|s| s.to_string()); }
    if let Some(v) = updates.get("support_requested").and_then(|v| v.as_bool()) { therapist.support_requested = v; }
    if let Some(v) = updates.get("onboarding_complete").and_then(|v| v.as_bool()) { therapist.onboarding_complete = v; }

    let updated = therapist_svc.update(&therapist).await?;
    Ok(HttpResponse::Ok().json(updated))
}

// ─── Availability Set ────────────────────────────────────────────────────────

pub async fn set_availability(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let data = body.into_inner();
    let day_of_week = data.get("day_of_week").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
    let start_time_str = data.get("start_time").and_then(|v| v.as_str()).unwrap_or("09:00");
    let end_time_str = data.get("end_time").and_then(|v| v.as_str()).unwrap_or("17:00");
    let is_active = data.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);

    let start_time = chrono::NaiveTime::parse_from_str(start_time_str, "%H:%M")
        .or_else(|_| chrono::NaiveTime::parse_from_str(start_time_str, "%H:%M:%S"))
        .map_err(|_| AppError::bad_request("Invalid start_time format"))?;
    let end_time = chrono::NaiveTime::parse_from_str(end_time_str, "%H:%M")
        .or_else(|_| chrono::NaiveTime::parse_from_str(end_time_str, "%H:%M:%S"))
        .map_err(|_| AppError::bad_request("Invalid end_time format"))?;

    let result = therapist_svc.set_availability(user.id, day_of_week, start_time, end_time, is_active).await?;
    Ok(HttpResponse::Ok().json(result))
}

// ─── Integration Disconnect ──────────────────────────────────────────────────

pub async fn disconnect_zoom(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    // Clear zoom tokens by updating therapist
    let mut therapist = therapist_svc.get_me(user.id).await?;
    therapist.zoom_connected = false;
    therapist_svc.update(&therapist).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

pub async fn disconnect_google(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let mut therapist = therapist_svc.get_me(user.id).await?;
    therapist.google_connected = false;
    therapist_svc.update(&therapist).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

// ─── Integration Status ──────────────────────────────────────────────────────

pub async fn integration_status(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_me(user.id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "zoom": therapist.zoom_connected,
        "google_calendar": therapist.google_connected,
    })))
}

// ─── Plan Selection ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SelectPlanBody {
    pub plan: String,
    /// Therapist's email, passed from the client so we can attach it to the lead.
    /// Optional — the frontend should send auth user email here.
    pub email: Option<String>,
}

pub async fn select_plan(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
    body: web::Json<SelectPlanBody>,
) -> Result<HttpResponse, AppError> {
    let updated = therapist_svc
        .select_plan(user.id, &body.plan, body.email.clone())
        .await?;
    Ok(HttpResponse::Ok().json(updated))
}

// ─── Complete Onboarding ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CompleteOnboardingBody {
    pub avatar_key: Option<String>,
    pub bio: Option<String>,
    pub support_requested: bool,
}

pub async fn complete_onboarding(
    user: AuthUser,
    therapist_svc: web::Data<TherapistService>,
    body: web::Json<CompleteOnboardingBody>,
) -> Result<HttpResponse, AppError> {
    let updated = therapist_svc
        .complete_onboarding(
            user.id,
            body.avatar_key.clone(),
            body.bio.clone(),
            body.support_requested,
        )
        .await?;
    Ok(HttpResponse::Ok().json(updated))
}

// ─── Route Configuration ─────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
// Therapist
            .route("/api/v1/therapists/me", web::get().to(get_me))
            .route("/api/v1/therapists/me", web::put().to(update_me))
            .route("/api/v1/therapists/me/select-plan", web::post().to(select_plan))
            .route("/api/v1/therapists/me/complete-onboarding", web::post().to(complete_onboarding))
            .route("/api/v1/therapists/me/availability", web::get().to(get_availability))
            .route("/api/v1/therapists/me/availability", web::put().to(set_availability))
            .route("/api/v1/therapists/by-slug/{slug}", web::get().to(get_by_slug))
            // Practice
            .route("/api/v1/practices/me", web::get().to(get_my_practice))
            .route("/api/v1/practices/{id}/members", web::get().to(list_members))
            // Onboarding
            .route("/api/v1/onboarding/tokens", web::get().to(list_onboarding_tokens))
            .route("/api/v1/onboarding/by-token/{token}", web::get().to(get_onboarding_token))
            // Integrations
            .route("/api/v1/integrations/status", web::get().to(integration_status))
            .route("/api/v1/integrations/zoom/disconnect", web::post().to(disconnect_zoom))
            .route("/api/v1/integrations/google/disconnect", web::post().to(disconnect_google));
}
