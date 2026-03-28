use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::scheduling::domain::entity::{SessionType, SessionTypeRate};
use crate::scheduling::domain::error::SchedulingError;
use crate::scheduling::domain::port::SessionTypeRepository;

pub struct PgSessionTypeRepository {
    pool: PgPool,
}

impl PgSessionTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionTypeRepository for PgSessionTypeRepository {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<SessionType>, SchedulingError> {
        sqlx::query_as::<_, SessionType>(
            "SELECT
                id, therapist_id, name, duration_mins, rate_inr,
                description, is_active, sort_order, intake_form_id,
                created_at, updated_at
            FROM session_types
            WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<SessionType>, SchedulingError> {
        sqlx::query_as::<_, SessionType>(
            "SELECT
                id, therapist_id, name, duration_mins, rate_inr,
                description, is_active, sort_order, intake_form_id,
                created_at, updated_at
            FROM session_types
            WHERE therapist_id = $1
            ORDER BY sort_order, name"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_active_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<SessionType>, SchedulingError> {
        sqlx::query_as::<_, SessionType>(
            "SELECT
                id, therapist_id, name, duration_mins, rate_inr,
                description, is_active, sort_order, intake_form_id,
                created_at, updated_at
            FROM session_types
            WHERE therapist_id = $1 AND is_active = true
            ORDER BY sort_order, name"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

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
    ) -> Result<SessionType, SchedulingError> {
        sqlx::query_as::<_, SessionType>(
            "INSERT INTO session_types (
                therapist_id, name, duration_mins, rate_inr,
                description, is_active, sort_order, intake_form_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, therapist_id, name, duration_mins, rate_inr,
                description, is_active, sort_order, intake_form_id,
                created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(name)
        .bind(duration_mins)
        .bind(rate_inr)
        .bind(description)
        .bind(is_active)
        .bind(sort_order)
        .bind(intake_form_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

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
    ) -> Result<SessionType, SchedulingError> {
        sqlx::query_as::<_, SessionType>(
            "UPDATE session_types SET
                name = $3,
                duration_mins = $4,
                rate_inr = $5,
                description = $6,
                is_active = $7,
                sort_order = $8,
                intake_form_id = $9,
                updated_at = now()
            WHERE id = $1 AND therapist_id = $2
            RETURNING
                id, therapist_id, name, duration_mins, rate_inr,
                description, is_active, sort_order, intake_form_id,
                created_at, updated_at"
        )
        .bind(id)
        .bind(therapist_id)
        .bind(name)
        .bind(duration_mins)
        .bind(rate_inr)
        .bind(description)
        .bind(is_active)
        .bind(sort_order)
        .bind(intake_form_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn delete(&self, therapist_id: Uuid, id: Uuid) -> Result<(), SchedulingError> {
        sqlx::query("DELETE FROM session_types WHERE id = $1 AND therapist_id = $2")
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| SchedulingError::Database(e.to_string()))?;
        Ok(())
    }

    async fn reorder(
        &self,
        therapist_id: Uuid,
        ordered_ids: &[Uuid],
    ) -> Result<(), SchedulingError> {
        for (index, id) in ordered_ids.iter().enumerate() {
            sqlx::query(
                "UPDATE session_types SET sort_order = $3, updated_at = now()
                WHERE id = $1 AND therapist_id = $2"
            )
            .bind(id)
            .bind(therapist_id)
            .bind(index as i32)
            .execute(&self.pool)
            .await
            .map_err(|e| SchedulingError::Database(e.to_string()))?;
        }
        Ok(())
    }

    async fn find_rates(
        &self,
        session_type_id: Uuid,
    ) -> Result<Vec<SessionTypeRate>, SchedulingError> {
        sqlx::query_as::<_, SessionTypeRate>(
            "SELECT id, session_type_id, client_category, rate_inr
            FROM session_type_rates
            WHERE session_type_id = $1
            ORDER BY client_category"
        )
        .bind(session_type_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }
}
