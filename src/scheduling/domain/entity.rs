use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Session ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,

    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub duration_mins: i32,
    pub status: String,

    pub zoom_meeting_id: Option<String>,
    pub zoom_join_url: Option<String>,
    pub zoom_start_url: Option<String>,
    pub google_event_id: Option<String>,

    pub payment_status: String,
    pub amount_inr: i32,
    pub razorpay_payment_id: Option<String>,

    pub reminder_24h_sent: bool,
    pub reminder_1h_sent: bool,

    pub session_number: Option<i32>,
    pub cancellation_reason: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<String>,
    pub is_late_cancellation: Option<bool>,

    pub session_type_name: Option<String>,
    pub recurring_reservation_id: Option<Uuid>,

    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Blocked Slot ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct BlockedSlot {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub reason: Option<String>,
}

// ─── Recurring Reservation ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct RecurringReservation {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,
    pub day_of_week: i32,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub session_type_name: Option<String>,
    pub amount_inr: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Session Type ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct SessionType {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub name: String,
    pub duration_mins: i32,
    pub rate_inr: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub intake_form_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Session Type Rate ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct SessionTypeRate {
    pub id: Uuid,
    pub session_type_id: Uuid,
    pub client_category: String,
    pub rate_inr: i32,
}

// ─── Time Slot (for available-slot responses) ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TimeSlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_mins: i32,
}
