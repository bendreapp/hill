use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use uuid::Uuid;

use super::entity::*;
use super::error::SchedulingError;

// ─── Session Repository ─────────────────────────────────────────────────────

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Session>, SchedulingError>;

    async fn list_by_date_range(
        &self,
        therapist_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SchedulingError>;

    async fn list_pending(&self, therapist_id: Uuid) -> Result<Vec<Session>, SchedulingError>;

    async fn list_today(
        &self,
        therapist_id: Uuid,
        day_start: DateTime<Utc>,
        day_end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SchedulingError>;

    async fn list_upcoming(
        &self,
        therapist_id: Uuid,
        from: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<Session>, SchedulingError>;

    async fn list_by_client(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<Session>, SchedulingError>;

    async fn create(&self, session: &Session) -> Result<Session, SchedulingError>;

    async fn update_status(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        status: &str,
        cancellation_reason: Option<&str>,
        cancelled_by: Option<&str>,
        is_late_cancellation: Option<bool>,
    ) -> Result<Session, SchedulingError>;

    async fn update(&self, session: &Session) -> Result<Session, SchedulingError>;

    async fn soft_delete(&self, therapist_id: Uuid, id: Uuid) -> Result<(), SchedulingError>;
}

// ─── Blocked Slot Repository ────────────────────────────────────────────────

#[async_trait]
pub trait BlockedSlotRepository: Send + Sync {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<BlockedSlot>, SchedulingError>;

    async fn list_by_range(
        &self,
        therapist_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<BlockedSlot>, SchedulingError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<BlockedSlot, SchedulingError>;

    async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<BlockedSlot, SchedulingError>;

    async fn delete(&self, therapist_id: Uuid, id: Uuid) -> Result<(), SchedulingError>;
}

// ─── Recurring Reservation Repository ───────────────────────────────────────

#[async_trait]
pub trait RecurringReservationRepository: Send + Sync {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<RecurringReservation>, SchedulingError>;

    async fn list_active(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<RecurringReservation>, SchedulingError>;

    async fn list_by_client(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<RecurringReservation>, SchedulingError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        day_of_week: i32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        session_type_name: Option<&str>,
        amount_inr: i32,
    ) -> Result<RecurringReservation, SchedulingError>;

    async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        day_of_week: i32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        session_type_name: Option<&str>,
        amount_inr: i32,
    ) -> Result<RecurringReservation, SchedulingError>;

    async fn deactivate(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<RecurringReservation, SchedulingError>;
}

// ─── Session Type Repository ────────────────────────────────────────────────

#[async_trait]
pub trait SessionTypeRepository: Send + Sync {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<SessionType>, SchedulingError>;

    async fn list_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<SessionType>, SchedulingError>;

    async fn list_active_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<SessionType>, SchedulingError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        name: &str,
        duration_mins: i32,
        rate_inr: i32,
        description: Option<&str>,
        is_active: bool,
        sort_order: i32,
        intake_form_id: Option<Uuid>,
    ) -> Result<SessionType, SchedulingError>;

    async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        name: &str,
        duration_mins: i32,
        rate_inr: i32,
        description: Option<&str>,
        is_active: bool,
        sort_order: i32,
        intake_form_id: Option<Uuid>,
    ) -> Result<SessionType, SchedulingError>;

    async fn delete(&self, therapist_id: Uuid, id: Uuid) -> Result<(), SchedulingError>;

    async fn reorder(
        &self,
        therapist_id: Uuid,
        ordered_ids: &[Uuid],
    ) -> Result<(), SchedulingError>;

    async fn find_rates(
        &self,
        session_type_id: Uuid,
    ) -> Result<Vec<SessionTypeRate>, SchedulingError>;
}
