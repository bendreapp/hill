use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::clinical::application::service::{MessageService, NoteService, TreatmentPlanService};
use crate::clinical::domain::entity::{
    CreateMessageInput, CreateNoteInput, CreateTreatmentPlanInput, MarkReadInput,
    UpdateNoteInput, UpdateTreatmentPlanInput,
};
use crate::shared::error::AppError;
use crate::shared::types::{AuthUser, Paginated};

// ─── Query Parameters ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ThreadQuery {
    pub client_id: Uuid,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ─── Note Handlers ──────────────────────────────────────────────────────────

pub async fn list_notes(
    user: AuthUser,
    svc: web::Data<NoteService>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc.list_notes(&[user.id], per_page, offset).await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_note(
    user: AuthUser,
    svc: web::Data<NoteService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let note = svc.get_note(*id, user.id).await?;
    Ok(HttpResponse::Ok().json(note))
}

pub async fn get_note_by_session(
    user: AuthUser,
    svc: web::Data<NoteService>,
    session_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let note = svc.get_note_by_session(*session_id, user.id).await?;
    Ok(HttpResponse::Ok().json(note))
}

pub async fn create_note(
    user: AuthUser,
    svc: web::Data<NoteService>,
    body: web::Json<CreateNoteInput>,
) -> Result<HttpResponse, AppError> {
    let note = svc.create_note(user.id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(note))
}

pub async fn update_note(
    user: AuthUser,
    svc: web::Data<NoteService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateNoteInput>,
) -> Result<HttpResponse, AppError> {
    let note = svc.update_note(*id, user.id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(note))
}

pub async fn delete_note(
    user: AuthUser,
    svc: web::Data<NoteService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.delete_note(*id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ─── Treatment Plan Handlers ────────────────────────────────────────────────

pub async fn list_plans(
    user: AuthUser,
    svc: web::Data<TreatmentPlanService>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc.list_by_therapist(&[user.id], per_page, offset).await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn list_plans_by_client(
    user: AuthUser,
    svc: web::Data<TreatmentPlanService>,
    client_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc.list_by_client(*client_id, user.id, per_page, offset).await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_plan(
    user: AuthUser,
    svc: web::Data<TreatmentPlanService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let plan = svc.get_plan(*id, user.id).await?;
    Ok(HttpResponse::Ok().json(plan))
}

pub async fn create_plan(
    user: AuthUser,
    svc: web::Data<TreatmentPlanService>,
    body: web::Json<CreateTreatmentPlanInput>,
) -> Result<HttpResponse, AppError> {
    let plan = svc.create_plan(user.id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(plan))
}

pub async fn update_plan(
    user: AuthUser,
    svc: web::Data<TreatmentPlanService>,
    id: web::Path<Uuid>,
    body: web::Json<UpdateTreatmentPlanInput>,
) -> Result<HttpResponse, AppError> {
    let plan = svc.update_plan(*id, user.id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(plan))
}

pub async fn delete_plan(
    user: AuthUser,
    svc: web::Data<TreatmentPlanService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.delete_plan(*id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ─── Message Handlers ───────────────────────────────────────────────────────

pub async fn list_thread(
    user: AuthUser,
    svc: web::Data<MessageService>,
    query: web::Query<ThreadQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc
        .list_thread(user.id, query.client_id, per_page, offset)
        .await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn send_message(
    user: AuthUser,
    svc: web::Data<MessageService>,
    body: web::Json<CreateMessageInput>,
) -> Result<HttpResponse, AppError> {
    let msg = svc.send_message(user.id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(msg))
}

pub async fn mark_read(
    user: AuthUser,
    svc: web::Data<MessageService>,
    body: web::Json<MarkReadInput>,
) -> Result<HttpResponse, AppError> {
    svc.mark_read(user.id, &body.message_ids).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn unread_count(
    user: AuthUser,
    svc: web::Data<MessageService>,
) -> Result<HttpResponse, AppError> {
    let count = svc.unread_count(user.id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
            // Session notes
            .route("/api/v1/notes", web::get().to(list_notes))
            .route("/api/v1/notes", web::post().to(create_note))
            .route("/api/v1/notes/{id}", web::get().to(get_note))
            .route("/api/v1/notes/{id}", web::put().to(update_note))
            .route("/api/v1/notes/{id}", web::delete().to(delete_note))
            .route("/api/v1/sessions/{session_id}/note", web::get().to(get_note_by_session))
            // Treatment plans
            .route("/api/v1/treatment-plans", web::get().to(list_plans))
            .route("/api/v1/treatment-plans", web::post().to(create_plan))
            .route("/api/v1/treatment-plans/{id}", web::get().to(get_plan))
            .route("/api/v1/treatment-plans/{id}", web::put().to(update_plan))
            .route("/api/v1/treatment-plans/{id}", web::delete().to(delete_plan))
            .route("/api/v1/clients/{client_id}/treatment-plans", web::get().to(list_plans_by_client))
            // Messages
            .route("/api/v1/messages", web::get().to(list_thread))
            .route("/api/v1/messages", web::post().to(send_message))
            .route("/api/v1/messages/read", web::post().to(mark_read))
            .route("/api/v1/messages/unread-count", web::get().to(unread_count));
}
