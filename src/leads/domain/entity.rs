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
    pub converted_client_id: Option<Uuid>,
    pub notes: Option<String>,
    pub preferred_times: Option<serde_json::Value>,
    pub message: Option<String>,
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
    /// Free-text preferred time slots, e.g. ["April 10, 3pm", "weekday evenings"]
    pub preferred_times: Option<Vec<String>>,
    /// "What brings you here?" message from the public inquiry form
    pub message: Option<String>,
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
    pub invite_sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ─── Convert Lead to Client Response ───────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ConvertLeadResponse {
    pub client_id: Uuid,
    pub lead_id: Uuid,
    pub status: String,
}

// ─── Send Portal Invite Response ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SendPortalInviteResponse {
    pub invitation_id: Uuid,
    pub token: String,
    pub sent_at: DateTime<Utc>,
}
