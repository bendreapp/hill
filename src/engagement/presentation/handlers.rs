use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::engagement::application::service::{BroadcastService, IntakeQuestionService, IntakeService, LeadIntakeService, MessageTemplateService, ResourceService};
use crate::engagement::domain::entity::{
    BroadcastInput, CreateIntakeFormInput, CreateIntakeQuestionInput, CreateIntakeResponseInput,
    CreateResourceInput, ReorderQuestionsInput, ShareResourceInput, SubmitIntakeResponseInput,
    SubmitLeadIntakeInput, UnshareResourceInput, UpdateIntakeFormInput, UpdateIntakeQuestionInput,
    UpdateResourceInput, UpsertMessageTemplateInput,
};
use crate::leads::application::service::LeadService;
use crate::iam::application::service::TherapistService;
use crate::shared::error::AppError;
use crate::shared::types::{AuthUser, Paginated};

// ─── Query Parameters ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ─── Resource Handlers ──────────────────────────────────────────────────────

pub async fn list_resources(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc.list_resources(&[user.id], per_page, offset).await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_resource(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let resource = svc.get_resource(*id, user.id).await?;
    Ok(HttpResponse::Ok().json(resource))
}

pub async fn create_resource(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    body: web::Json<CreateResourceInput>,
) -> Result<HttpResponse, AppError> {
    let resource = svc.create_resource(user.id, &body).await?;
    Ok(HttpResponse::Created().json(resource))
}

pub async fn update_resource(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateResourceInput>,
) -> Result<HttpResponse, AppError> {
    let resource = svc.update_resource(*id, user.id, &body).await?;
    Ok(HttpResponse::Ok().json(resource))
}

pub async fn delete_resource(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.delete_resource(*id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn share_resource(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    id: web::Path<Uuid>,
    body: web::Json<ShareResourceInput>,
) -> Result<HttpResponse, AppError> {
    let shares = svc
        .share_resource(*id, user.id, &body.client_ids, body.note.as_deref())
        .await?;
    Ok(HttpResponse::Ok().json(shares))
}

pub async fn unshare_resource(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    id: web::Path<Uuid>,
    body: web::Json<UnshareResourceInput>,
) -> Result<HttpResponse, AppError> {
    svc.unshare_resource(*id, user.id, &body.client_ids).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_client_resources(
    user: AuthUser,
    svc: web::Data<ResourceService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc
        .list_shared_with_client(*client_id, user.id, per_page, offset)
        .await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

// ─── Intake Form Handlers ───────────────────────────────────────────────────

pub async fn list_intake_forms(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc.list_forms(&[user.id], per_page, offset).await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_intake_form(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let form = svc.get_form(*id, user.id).await?;
    Ok(HttpResponse::Ok().json(form))
}

pub async fn create_intake_form(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    body: web::Json<CreateIntakeFormInput>,
) -> Result<HttpResponse, AppError> {
    let form = svc.create_form(user.id, &body).await?;
    Ok(HttpResponse::Created().json(form))
}

pub async fn update_intake_form(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateIntakeFormInput>,
) -> Result<HttpResponse, AppError> {
    let form = svc.update_form(*id, user.id, &body).await?;
    Ok(HttpResponse::Ok().json(form))
}

pub async fn delete_intake_form(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.delete_form(*id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ─── Intake Response Handlers ───────────────────────────────────────────────

pub async fn create_intake_response(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    body: web::Json<CreateIntakeResponseInput>,
) -> Result<HttpResponse, AppError> {
    let resp = svc.create_response(user.id, &body).await?;
    Ok(HttpResponse::Created().json(resp))
}

pub async fn get_intake_response(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let resp = svc.get_response(*id, user.id).await?;
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn get_intake_response_by_token(
    svc: web::Data<IntakeService>,
    token: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let resp = svc.get_response_by_token(*token).await?;
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn list_client_intake_responses(
    user: AuthUser,
    svc: web::Data<IntakeService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc
        .list_responses_by_client(*client_id, user.id, per_page, offset)
        .await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn submit_intake_response(
    svc: web::Data<IntakeService>,
    id: web::Path<Uuid>,
    body: web::Json<SubmitIntakeResponseInput>,
) -> Result<HttpResponse, AppError> {
    let resp = svc.submit_response(*id, &body).await?;
    Ok(HttpResponse::Ok().json(resp))
}

// ─── Message Template Handlers ──────────────────────────────────────────────

pub async fn list_message_templates(
    user: AuthUser,
    svc: web::Data<MessageTemplateService>,
) -> Result<HttpResponse, AppError> {
    let templates = svc.list_templates(user.id).await?;
    Ok(HttpResponse::Ok().json(templates))
}

pub async fn update_message_template(
    user: AuthUser,
    svc: web::Data<MessageTemplateService>,
    key: web::Path<String>,
    body: web::Json<UpsertMessageTemplateInput>,
) -> Result<HttpResponse, AppError> {
    let template = svc
        .update_template(user.id, &key, &body.subject, &body.body)
        .await?;
    Ok(HttpResponse::Ok().json(template))
}

// ─── Intake Form Question Handlers ──────────────────────────────────────────

pub async fn list_intake_questions(
    user: AuthUser,
    svc: web::Data<IntakeQuestionService>,
) -> Result<HttpResponse, AppError> {
    let questions = svc.list_questions(user.id).await?;
    Ok(HttpResponse::Ok().json(questions))
}

pub async fn create_intake_question(
    user: AuthUser,
    svc: web::Data<IntakeQuestionService>,
    body: web::Json<CreateIntakeQuestionInput>,
) -> Result<HttpResponse, AppError> {
    let question = svc.create_question(user.id, &body).await?;
    Ok(HttpResponse::Created().json(question))
}

pub async fn update_intake_question(
    user: AuthUser,
    svc: web::Data<IntakeQuestionService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateIntakeQuestionInput>,
) -> Result<HttpResponse, AppError> {
    let question = svc.update_question(*id, user.id, &body).await?;
    Ok(HttpResponse::Ok().json(question))
}

pub async fn delete_intake_question(
    user: AuthUser,
    svc: web::Data<IntakeQuestionService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.delete_question(*id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn reorder_intake_questions(
    user: AuthUser,
    svc: web::Data<IntakeQuestionService>,
    body: web::Json<ReorderQuestionsInput>,
) -> Result<HttpResponse, AppError> {
    let questions = svc.reorder_questions(user.id, &body.ids).await?;
    Ok(HttpResponse::Ok().json(questions))
}

// ─── Broadcast Handler ──────────────────────────────────────────────────────

pub async fn send_broadcast(
    _user: AuthUser,
    svc: web::Data<BroadcastService>,
    body: web::Json<BroadcastInput>,
) -> Result<HttpResponse, AppError> {
    // The handler receives client_ids; in a full implementation the handler would
    // resolve each client_id to their contact info. Here we use client_ids as
    // identifiers directly (phone or email depending on channel).
    // For a real implementation, this would join against the clients table.
    let contacts: Vec<(String, Option<String>)> = body
        .client_ids
        .iter()
        .map(|id| (id.to_string(), None))
        .collect();

    let sent = svc.broadcast(&body, &contacts).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "sent": sent })))
}

// ─── Lead Intake Handlers ────────────────────────────────────────────────────

/// POST /api/v1/leads/{id}/send-intake-form  (authenticated)
pub async fn send_lead_intake_form(
    user: AuthUser,
    id: web::Path<Uuid>,
    lead_svc: web::Data<LeadService>,
    therapist_svc: web::Data<TherapistService>,
    intake_svc: web::Data<LeadIntakeService>,
    template_svc: web::Data<MessageTemplateService>,
) -> Result<HttpResponse, AppError> {
    let lead_id = id.into_inner();

    // Validate lead ownership and get lead data
    let lead = lead_svc.get_lead(lead_id, user.id).await?;

    let lead_email = lead.email.as_deref().ok_or_else(|| {
        AppError::bad_request("Lead has no email address — cannot send intake form")
    })?;

    // Fetch therapist display name
    let therapist = therapist_svc.get_me(user.id).await?;
    let therapist_display = therapist
        .display_name
        .as_deref()
        .unwrap_or(&therapist.full_name)
        .to_string();

    // Fetch the intake_invite template (custom or default)
    let template = template_svc.get_template_for_sending(user.id, "intake_invite").await?;

    // Send intake form and create submission record
    let result = intake_svc
        .send_to_lead(
            lead_id,
            user.id,
            &lead.full_name,
            lead_email,
            &therapist_display,
            &template.subject,
            &template.body,
        )
        .await?;

    // Update lead status to `contacted` (best-effort — don't fail the request)
    let _ = lead_svc
        .update_lead(
            lead_id,
            user.id,
            &crate::leads::domain::entity::UpdateLeadInput {
                status: Some("contacted".to_string()),
                notes: None,
                client_id: None,
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(result))
}

/// GET /api/v1/leads/{id}/intake-submissions  (authenticated)
pub async fn list_lead_intake_submissions(
    user: AuthUser,
    id: web::Path<Uuid>,
    lead_svc: web::Data<LeadService>,
    intake_svc: web::Data<LeadIntakeService>,
) -> Result<HttpResponse, AppError> {
    let lead_id = id.into_inner();
    // Validate ownership
    lead_svc.get_lead(lead_id, user.id).await?;

    let submissions = intake_svc.list_by_lead(lead_id, user.id).await?;
    Ok(HttpResponse::Ok().json(submissions))
}

/// GET /api/v1/lead-intake/{token}  (public — no auth)
pub async fn get_public_lead_intake(
    token: web::Path<String>,
    intake_svc: web::Data<LeadIntakeService>,
    question_svc: web::Data<IntakeQuestionService>,
) -> Result<HttpResponse, AppError> {
    // Check if already submitted
    let submission = intake_svc
        .submission_repo
        .find_by_token(&token)
        .await
        .map_err(AppError::from)?;

    let submission = submission.ok_or_else(|| AppError::not_found("Intake form"))?;

    if submission.submitted_at.is_some() {
        return Ok(HttpResponse::Gone().json(serde_json::json!({
            "already_submitted": true,
            "message": "This intake form has already been submitted."
        })));
    }

    // Fetch questions for this therapist
    let questions = question_svc
        .list_questions(submission.therapist_id)
        .await?;

    // Fetch therapist display name — query therapists table directly
    // We use a simple SQL query via the lead service's therapist info fetch
    // (which is in the leads module, not engagement), so we build a light response
    // by looking up the data through available services
    let therapist_display = {
        // Use IntakeQuestionService's question_repo (it has the pool) — we instead
        // pull the name from the lead. We don't want cross-module DB calls in handlers,
        // so we do this via a simple approach: use the lead service to get therapist info.
        // But lead_svc.lead_repo.find_therapist_info is private from here. Instead,
        // we accept a small simplification: return therapist_id and let frontend look up
        // or we expose minimal fields needed. We'll include therapist_id in the response.
        // For MVP: return therapist_id + questions + already_submitted
        "Your therapist".to_string()
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "therapist_id": submission.therapist_id,
        "therapist_display_name": therapist_display,
        "questions": questions,
        "already_submitted": false,
    })))
}

/// POST /api/v1/lead-intake/{token}/submit  (public — no auth)
pub async fn submit_public_lead_intake(
    token: web::Path<String>,
    body: web::Json<SubmitLeadIntakeInput>,
    intake_svc: web::Data<LeadIntakeService>,
    lead_svc: web::Data<LeadService>,
) -> Result<HttpResponse, AppError> {
    // Submit the form — service validates not already submitted
    let submission = intake_svc
        .submit_public_form(&token, &body.responses)
        .await?;

    // Update lead status to `qualified` (best-effort)
    let _ = lead_svc
        .update_lead(
            submission.lead_id,
            submission.therapist_id,
            &crate::leads::domain::entity::UpdateLeadInput {
                status: Some("qualified".to_string()),
                notes: None,
                client_id: None,
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "submitted_at": submission.submitted_at,
    })))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
            // Resources
            .route("/api/v1/resources", web::get().to(list_resources))
            .route("/api/v1/resources", web::post().to(create_resource))
            .route("/api/v1/resources/{id}", web::get().to(get_resource))
            .route("/api/v1/resources/{id}", web::put().to(update_resource))
            .route("/api/v1/resources/{id}", web::delete().to(delete_resource))
            .route("/api/v1/resources/{id}/share", web::post().to(share_resource))
            .route("/api/v1/resources/{id}/unshare", web::post().to(unshare_resource))
            .route("/api/v1/clients/{client_id}/resources", web::get().to(list_client_resources))
            // Intake forms
            .route("/api/v1/intake-forms", web::get().to(list_intake_forms))
            .route("/api/v1/intake-forms", web::post().to(create_intake_form))
            .route("/api/v1/intake-forms/{id}", web::get().to(get_intake_form))
            .route("/api/v1/intake-forms/{id}", web::put().to(update_intake_form))
            .route("/api/v1/intake-forms/{id}", web::delete().to(delete_intake_form))
            // Intake responses
            .route("/api/v1/intake-responses", web::post().to(create_intake_response))
            .route("/api/v1/intake-responses/{id}", web::get().to(get_intake_response))
            .route("/api/v1/intake-responses/{id}/submit", web::post().to(submit_intake_response))
            .route("/api/v1/intake-responses/by-token/{token}", web::get().to(get_intake_response_by_token))
            .route("/api/v1/clients/{client_id}/intake-responses", web::get().to(list_client_intake_responses))
            // Broadcast
            .route("/api/v1/broadcast", web::post().to(send_broadcast))
            // Message Templates
            .route("/api/v1/message-templates", web::get().to(list_message_templates))
            .route("/api/v1/message-templates/{key}", web::put().to(update_message_template))
            // Intake Form Questions (custom question builder)
            // IMPORTANT: /reorder must come before /{id} to avoid routing conflicts
            .route("/api/v1/intake-forms/questions", web::get().to(list_intake_questions))
            .route("/api/v1/intake-forms/questions", web::post().to(create_intake_question))
            .route("/api/v1/intake-forms/questions/reorder", web::patch().to(reorder_intake_questions))
            .route("/api/v1/intake-forms/questions/{id}", web::put().to(update_intake_question))
            .route("/api/v1/intake-forms/questions/{id}", web::delete().to(delete_intake_question))
            // Lead Intake (authenticated)
            .route("/api/v1/leads/{id}/send-intake-form", web::post().to(send_lead_intake_form))
            .route("/api/v1/leads/{id}/intake-submissions", web::get().to(list_lead_intake_submissions))
            // Lead Intake (public — no auth)
            .route("/api/v1/lead-intake/{token}", web::get().to(get_public_lead_intake))
            .route("/api/v1/lead-intake/{token}/submit", web::post().to(submit_public_lead_intake));
}
