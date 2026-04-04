use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Therapist Aggregate ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct Therapist {
    pub id: Uuid,
    pub slug: String,
    pub full_name: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub qualifications: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: String,

    pub session_duration_mins: i32,
    pub buffer_mins: i32,
    pub session_rate_inr: i32,

    pub booking_page_active: bool,
    pub show_pricing: bool,
    pub gstin: Option<String>,

    pub google_connected: bool,
    pub zoom_connected: bool,

    pub cancellation_hours: i32,
    pub min_booking_advance_hours: i32,
    pub no_show_charge_percent: i32,
    pub late_cancel_charge_percent: i32,

    pub cancellation_policy: Option<String>,
    pub late_policy: Option<String>,
    pub rescheduling_policy: Option<String>,

    pub custom_tags: Option<serde_json::Value>,
    pub practice_id: Option<Uuid>,

    // ── Onboarding / Plan fields ──────────────────────────────────────────────
    pub whatsapp_number: Option<String>,
    pub team_size: Option<i32>,
    pub comms_whatsapp: bool,
    pub comms_email: bool,
    pub comms_sms: bool,
    pub plan_selected: Option<String>,
    pub plan_status: String,
    pub support_requested: bool,
    pub onboarding_complete: bool,
    pub avatar_key: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Availability ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct Availability {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_active: bool,
}

// ─── Practice Aggregate ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct Practice {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct PracticeMember {
    pub id: Uuid,
    pub practice_id: Uuid,
    pub user_id: Uuid,
    pub therapist_id: Option<Uuid>,
    pub role: String,
    pub can_view_notes: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct PracticeInvitation {
    pub id: Uuid,
    pub practice_id: Uuid,
    pub invited_by: Uuid,
    pub token: Uuid,
    pub email: Option<String>,
    pub role: String,
    pub can_view_notes: bool,
    pub status: String,
    pub accepted_by: Option<Uuid>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ─── Onboarding Token ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct OnboardingToken {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub token: Uuid,
    pub label: Option<String>,
    pub is_active: bool,
    pub max_uses: Option<i32>,
    pub use_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
