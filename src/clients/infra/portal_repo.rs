use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::domain::entity::{ClientPortalProfile, PortalSession};
use crate::clients::domain::error::ClientError;
use crate::clients::domain::port::ClientPortalRepository;

pub struct PgClientPortalRepository {
    pool: PgPool,
}

impl PgClientPortalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientPortalRepository for PgClientPortalRepository {
    async fn list_profiles_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ClientPortalProfile>, ClientError> {
        sqlx::query_as::<_, ClientPortalProfile>(
            "SELECT
                id, therapist_id, full_name, email, phone,
                intake_completed,
                status::text as status
            FROM clients
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn upcoming_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PortalSession>, ClientError> {
        sqlx::query_as::<_, PortalSession>(
            "SELECT
                id,
                starts_at,
                ends_at,
                duration_mins,
                status::text as status
            FROM sessions
            WHERE client_id = $1
              AND starts_at > now()
              AND status::text = 'scheduled'
              AND deleted_at IS NULL
            ORDER BY starts_at ASC
            LIMIT $2"
        )
        .bind(client_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn past_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PortalSession>, i64), ClientError> {
        let rows = sqlx::query_as::<_, PortalSession>(
            "SELECT
                id,
                starts_at,
                ends_at,
                duration_mins,
                status::text as status
            FROM sessions
            WHERE client_id = $1
              AND (starts_at <= now() OR status::text != 'scheduled')
              AND deleted_at IS NULL
            ORDER BY starts_at DESC
            LIMIT $2 OFFSET $3"
        )
        .bind(client_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM sessions
            WHERE client_id = $1
              AND (starts_at <= now() OR status::text != 'scheduled')
              AND deleted_at IS NULL"
        )
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        Ok((rows, total))
    }
}
