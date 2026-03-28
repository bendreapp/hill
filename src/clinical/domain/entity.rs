use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Session Note ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct SessionNote {
    pub id: Uuid,
    pub session_id: Uuid,
    pub therapist_id: Uuid,
    pub note_type: String,
    pub subjective: Option<String>,
    pub objective: Option<String>,
    pub assessment: Option<String>,
    pub plan: Option<String>,
    pub freeform_content: Option<String>,
    pub homework: Option<String>,
    /// Encrypted field — plaintext after decryption in application layer.
    pub techniques_used: Option<String>,
    /// Encrypted field — plaintext after decryption in application layer.
    pub risk_flags: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateNoteInput {
    pub session_id: Uuid,
    pub note_type: Option<String>,
    pub subjective: Option<String>,
    pub objective: Option<String>,
    pub assessment: Option<String>,
    pub plan: Option<String>,
    pub freeform_content: Option<String>,
    pub homework: Option<String>,
    pub techniques_used: Option<String>,
    pub risk_flags: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateNoteInput {
    pub note_type: Option<String>,
    pub subjective: Option<String>,
    pub objective: Option<String>,
    pub assessment: Option<String>,
    pub plan: Option<String>,
    pub freeform_content: Option<String>,
    pub homework: Option<String>,
    pub techniques_used: Option<String>,
    pub risk_flags: Option<String>,
}

// ─── Treatment Plan ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct TreatmentPlan {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,
    pub title: String,
    pub modality: String,
    pub modality_other: Option<String>,
    pub presenting_concerns: Option<String>,
    pub diagnosis: Option<String>,
    /// Encrypted field — plaintext JSON after decryption in application layer.
    pub goals: Option<String>,
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateTreatmentPlanInput {
    pub client_id: Uuid,
    pub title: Option<String>,
    pub modality: Option<String>,
    pub modality_other: Option<String>,
    pub presenting_concerns: Option<String>,
    pub diagnosis: Option<String>,
    pub goals: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateTreatmentPlanInput {
    pub title: Option<String>,
    pub modality: Option<String>,
    pub modality_other: Option<String>,
    pub presenting_concerns: Option<String>,
    pub diagnosis: Option<String>,
    pub goals: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

// ─── Message ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Message {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,
    pub sender_type: String,
    /// Encrypted field — plaintext after decryption in application layer.
    pub content: String,
    pub read_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateMessageInput {
    pub client_id: Uuid,
    pub sender_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct MarkReadInput {
    pub message_ids: Vec<Uuid>,
}
