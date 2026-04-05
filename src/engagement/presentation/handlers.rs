use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::engagement::application::service::{BroadcastService, IntakeQuestionService, IntakeService, MessageTemplateService, ResourceService};
use crate::engagement::domain::entity::{
    BroadcastInput, CreateIntakeFormInput, CreateIntakeQuestionInput, CreateIntakeResponseInput,
    CreateResourceInput, ReorderQuestionsInput, ShareResourceInput, SubmitIntakeResponseInput,
    UnshareResourceInput, UpdateIntakeFormInput, UpdateIntakeQuestionInput, UpdateResourceInput,
    UpsertMessageTemplateInput,
};
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
            .route("/api/v1/intake-forms/questions/{id}", web::delete().to(delete_intake_question));
}
