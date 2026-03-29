use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Lead ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lead {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub reason: Option<String>,
    pub source: String,
    pub status: String,
    pub session_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLeadInput {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub reason: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLeadInput {
    pub status: Option<String>,
    pub notes: Option<String>,
    pub client_id: Option<Uuid>,
}

// ─── Client Invitation ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClientInvitation {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,
    pub token: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
