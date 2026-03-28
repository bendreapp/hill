use async_trait::async_trait;
use chrono::NaiveTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::scheduling::domain::entity::RecurringReservation;
use crate::scheduling::domain::error::SchedulingError;
use crate::scheduling::domain::port::RecurringReservationRepository;

pub struct PgRecurringReservationRepository {
    pool: PgPool,
}

impl PgRecurringReservationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecurringReservationRepository for PgRecurringReservationRepository {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<RecurringReservation>, SchedulingError> {
        sqlx::query_as::<_, RecurringReservation>(
            "SELECT
                id, therapist_id, client_id,
                day_of_week, start_time, end_time,
                session_type_name, amount_inr, is_active,
                created_at, updated_at
            FROM recurring_reservations
            WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_active(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<RecurringReservation>, SchedulingError> {
        sqlx::query_as::<_, RecurringReservation>(
            "SELECT
                id, therapist_id, client_id,
                day_of_week, start_time, end_time,
                session_type_name, amount_inr, is_active,
                created_at, updated_at
            FROM recurring_reservations
            WHERE therapist_id = $1 AND is_active = true
            ORDER BY day_of_week, start_time"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_by_client(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<RecurringReservation>, SchedulingError> {
        sqlx::query_as::<_, RecurringReservation>(
            "SELECT
                id, therapist_id, client_id,
                day_of_week, start_time, end_time,
                session_type_name, amount_inr, is_active,
                created_at, updated_at
            FROM recurring_reservations
            WHERE therapist_id = $1 AND client_id = $2
            ORDER BY day_of_week, start_time"
        )
        .bind(therapist_id)
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        day_of_week: i32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        session_type_name: Option<&str>,
        amount_inr: i32,
    ) -> Result<RecurringReservation, SchedulingError> {
        sqlx::query_as::<_, RecurringReservation>(
            "INSERT INTO recurring_reservations (
                therapist_id, client_id, day_of_week,
                start_time, end_time, session_type_name, amount_inr
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, therapist_id, client_id,
                day_of_week, start_time, end_time,
                session_type_name, amount_inr, is_active,
                created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(client_id)
        .bind(day_of_week)
        .bind(start_time)
        .bind(end_time)
        .bind(session_type_name)
        .bind(amount_inr)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        day_of_week: i32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        session_type_name: Option<&str>,
        amount_inr: i32,
    ) -> Result<RecurringReservation, SchedulingError> {
        sqlx::query_as::<_, RecurringReservation>(
            "UPDATE recurring_reservations SET
                day_of_week = $3,
                start_time = $4,
                end_time = $5,
                session_type_name = $6,
                amount_inr = $7,
                updated_at = now()
            WHERE id = $1 AND therapist_id = $2
            RETURNING
                id, therapist_id, client_id,
                day_of_week, start_time, end_time,
                session_type_name, amount_inr, is_active,
                created_at, updated_at"
        )
        .bind(id)
        .bind(therapist_id)
        .bind(day_of_week)
        .bind(start_time)
        .bind(end_time)
        .bind(session_type_name)
        .bind(amount_inr)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn deactivate(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<RecurringReservation, SchedulingError> {
        sqlx::query_as::<_, RecurringReservation>(
            "UPDATE recurring_reservations SET
                is_active = false,
                updated_at = now()
            WHERE id = $1 AND therapist_id = $2
            RETURNING
                id, therapist_id, client_id,
                day_of_week, start_time, end_time,
                session_type_name, amount_inr, is_active,
                created_at, updated_at"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }
}
