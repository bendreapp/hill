use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::scheduling::domain::entity::Session;
use crate::scheduling::domain::error::SchedulingError;
use crate::scheduling::domain::port::SessionRepository;

pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Session>, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "SELECT
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at
            FROM sessions
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_by_date_range(
        &self,
        therapist_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "SELECT
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at
            FROM sessions
            WHERE therapist_id = $1
              AND starts_at >= $2
              AND starts_at <= $3
              AND deleted_at IS NULL
            ORDER BY starts_at"
        )
        .bind(therapist_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_pending(&self, therapist_id: Uuid) -> Result<Vec<Session>, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "SELECT
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at
            FROM sessions
            WHERE therapist_id = $1
              AND status = 'pending_approval'
              AND deleted_at IS NULL
            ORDER BY starts_at"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_today(
        &self,
        therapist_id: Uuid,
        day_start: DateTime<Utc>,
        day_end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "SELECT
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at
            FROM sessions
            WHERE therapist_id = $1
              AND starts_at >= $2
              AND starts_at <= $3
              AND status IN ('scheduled', 'pending_approval')
              AND deleted_at IS NULL
            ORDER BY starts_at"
        )
        .bind(therapist_id)
        .bind(day_start)
        .bind(day_end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_upcoming(
        &self,
        therapist_id: Uuid,
        from: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<Session>, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "SELECT
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at
            FROM sessions
            WHERE therapist_id = $1
              AND starts_at >= $2
              AND status = 'scheduled'
              AND deleted_at IS NULL
            ORDER BY starts_at
            LIMIT $3"
        )
        .bind(therapist_id)
        .bind(from)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_by_client(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<Session>, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "SELECT
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at
            FROM sessions
            WHERE therapist_id = $1 AND client_id = $2 AND deleted_at IS NULL
            ORDER BY starts_at DESC"
        )
        .bind(therapist_id)
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn create(&self, session: &Session) -> Result<Session, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "INSERT INTO sessions (
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status, payment_status, amount_inr,
                session_type_name, recurring_reservation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::session_status, $8::payment_status, $9, $10, $11)
            RETURNING
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at"
        )
        .bind(session.id)
        .bind(session.therapist_id)
        .bind(session.client_id)
        .bind(session.starts_at)
        .bind(session.ends_at)
        .bind(session.duration_mins)
        .bind(&session.status)
        .bind(&session.payment_status)
        .bind(session.amount_inr)
        .bind(&session.session_type_name)
        .bind(session.recurring_reservation_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn update_status(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        status: &str,
        cancellation_reason: Option<&str>,
        cancelled_by: Option<&str>,
        is_late_cancellation: Option<bool>,
    ) -> Result<Session, SchedulingError> {
        let cancelled_at = if status == "cancelled" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query_as::<_, Session>(
            "UPDATE sessions SET
                status = $3::session_status,
                cancellation_reason = COALESCE($4, cancellation_reason),
                cancelled_by = COALESCE($5::cancellation_actor, cancelled_by),
                is_late_cancellation = COALESCE($6, is_late_cancellation),
                cancelled_at = COALESCE($7, cancelled_at),
                updated_at = now()
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL
            RETURNING
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at"
        )
        .bind(id)
        .bind(therapist_id)
        .bind(status)
        .bind(cancellation_reason)
        .bind(cancelled_by)
        .bind(is_late_cancellation)
        .bind(cancelled_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn update(&self, session: &Session) -> Result<Session, SchedulingError> {
        sqlx::query_as::<_, Session>(
            "UPDATE sessions SET
                starts_at = $3,
                ends_at = $4,
                duration_mins = $5,
                status = $6::session_status,
                payment_status = $7::payment_status,
                amount_inr = $8,
                session_type_name = $9,
                recurring_reservation_id = $10,
                zoom_meeting_id = $11,
                zoom_join_url = $12,
                zoom_start_url = $13,
                google_event_id = $14,
                razorpay_payment_id = $15,
                session_number = $16,
                updated_at = now()
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL
            RETURNING
                id, therapist_id, client_id,
                starts_at, ends_at, duration_mins,
                status::text as status,
                zoom_meeting_id, zoom_join_url, zoom_start_url, google_event_id,
                payment_status::text as payment_status,
                amount_inr, razorpay_payment_id,
                reminder_24h_sent, reminder_1h_sent,
                session_number, cancellation_reason, cancelled_at,
                cancelled_by::text as cancelled_by,
                is_late_cancellation,
                session_type_name, recurring_reservation_id,
                deleted_at, created_at, updated_at"
        )
        .bind(session.id)
        .bind(session.therapist_id)
        .bind(session.starts_at)
        .bind(session.ends_at)
        .bind(session.duration_mins)
        .bind(&session.status)
        .bind(&session.payment_status)
        .bind(session.amount_inr)
        .bind(&session.session_type_name)
        .bind(session.recurring_reservation_id)
        .bind(&session.zoom_meeting_id)
        .bind(&session.zoom_join_url)
        .bind(&session.zoom_start_url)
        .bind(&session.google_event_id)
        .bind(&session.razorpay_payment_id)
        .bind(session.session_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn soft_delete(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<(), SchedulingError> {
        sqlx::query(
            "UPDATE sessions SET deleted_at = now(), updated_at = now()
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(therapist_id)
        .execute(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))?;
        Ok(())
    }
}
