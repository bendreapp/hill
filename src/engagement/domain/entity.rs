use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Resource ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Resource {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub file_url: Option<String>,
    pub external_url: Option<String>,
    pub modality_tags: Option<Vec<String>>,
    pub category_tags: Option<Vec<String>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateResourceInput {
    pub title: String,
    pub description: Option<String>,
    pub resource_type: Option<String>,
    pub file_url: Option<String>,
    pub external_url: Option<String>,
    pub modality_tags: Option<Vec<String>>,
    pub category_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateResourceInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource_type: Option<String>,
    pub file_url: Option<String>,
    pub external_url: Option<String>,
    pub modality_tags: Option<Vec<String>>,
    pub category_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ClientResource {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub client_id: Uuid,
    pub therapist_id: Uuid,
    pub shared_at: DateTime<Utc>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ShareResourceInput {
    pub client_ids: Vec<Uuid>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UnshareResourceInput {
    pub client_ids: Vec<Uuid>,
}

// ─── Intake Form ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct IntakeForm {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub form_type: String,
    pub status: String,
    pub fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeFormInput {
    pub name: String,
    pub description: Option<String>,
    pub form_type: Option<String>,
    pub status: Option<String>,
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateIntakeFormInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub form_type: Option<String>,
    pub status: Option<String>,
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct IntakeResponse {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,
    pub intake_form_id: Uuid,
    pub session_id: Option<Uuid>,
    pub access_token: Uuid,
    pub status: String,
    /// Encrypted field — plaintext JSON after decryption in application layer.
    pub responses: Option<String>,
    pub form_snapshot: serde_json::Value,
    pub submitted_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeResponseInput {
    pub client_id: Uuid,
    pub intake_form_id: Uuid,
    pub session_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct SubmitIntakeResponseInput {
    pub responses: String,
}

// ─── Intake Form Question ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct IntakeFormQuestion {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub question_text: String,
    pub field_type: String,
    pub options: Option<serde_json::Value>,
    pub is_required: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeQuestionInput {
    pub question_text: String,
    pub field_type: String,
    pub options: Option<serde_json::Value>,
    pub is_required: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateIntakeQuestionInput {
    pub question_text: Option<String>,
    pub field_type: Option<String>,
    pub options: Option<serde_json::Value>,
    pub is_required: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ReorderQuestionsInput {
    pub ids: Vec<Uuid>,
}

// ─── Message Templates ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct MessageTemplate {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub template_key: String,
    pub subject: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpsertMessageTemplateInput {
    pub subject: String,
    pub body: String,
}

// ─── Broadcast ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct BroadcastInput {
    pub client_ids: Vec<Uuid>,
    pub channel: String,
    pub subject: Option<String>,
    pub body: String,
}
