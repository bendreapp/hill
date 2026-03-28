use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::scheduling::domain::entity::BlockedSlot;
use crate::scheduling::domain::error::SchedulingError;
use crate::scheduling::domain::port::BlockedSlotRepository;

pub struct PgBlockedSlotRepository {
    pool: PgPool,
}

impl PgBlockedSlotRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BlockedSlotRepository for PgBlockedSlotRepository {
    async fn find_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Option<BlockedSlot>, SchedulingError> {
        sqlx::query_as::<_, BlockedSlot>(
            "SELECT id, therapist_id, start_at, end_at, reason
            FROM blocked_slots
            WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn list_by_range(
        &self,
        therapist_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<BlockedSlot>, SchedulingError> {
        sqlx::query_as::<_, BlockedSlot>(
            "SELECT id, therapist_id, start_at, end_at, reason
            FROM blocked_slots
            WHERE therapist_id = $1
              AND start_at < $3
              AND end_at > $2
            ORDER BY start_at"
        )
        .bind(therapist_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<BlockedSlot, SchedulingError> {
        sqlx::query_as::<_, BlockedSlot>(
            "INSERT INTO blocked_slots (therapist_id, start_at, end_at, reason)
            VALUES ($1, $2, $3, $4)
            RETURNING id, therapist_id, start_at, end_at, reason"
        )
        .bind(therapist_id)
        .bind(start_at)
        .bind(end_at)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<BlockedSlot, SchedulingError> {
        sqlx::query_as::<_, BlockedSlot>(
            "UPDATE blocked_slots SET
                start_at = $3, end_at = $4, reason = $5
            WHERE id = $1 AND therapist_id = $2
            RETURNING id, therapist_id, start_at, end_at, reason"
        )
        .bind(id)
        .bind(therapist_id)
        .bind(start_at)
        .bind(end_at)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SchedulingError::Database(e.to_string()))
    }

    async fn delete(&self, therapist_id: Uuid, id: Uuid) -> Result<(), SchedulingError> {
        sqlx::query("DELETE FROM blocked_slots WHERE id = $1 AND therapist_id = $2")
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| SchedulingError::Database(e.to_string()))?;
        Ok(())
    }
}
