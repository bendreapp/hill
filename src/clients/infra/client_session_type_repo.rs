use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::domain::entity::{
    ClientSessionType, CreateClientSessionTypeInput, UpdateClientSessionTypeInput,
};
use crate::clients::domain::error::ClientError;
use crate::clients::domain::port::ClientSessionTypeRepository;

pub struct PgClientSessionTypeRepository {
    pool: PgPool,
}

impl PgClientSessionTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientSessionTypeRepository for PgClientSessionTypeRepository {
    async fn list_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Vec<ClientSessionType>, ClientError> {
        sqlx::query_as::<_, ClientSessionType>(
            "SELECT id, therapist_id, client_id, name, duration_mins, rate_inr,
                    mode, description, is_active, is_default, created_at, updated_at
             FROM client_session_types
             WHERE client_id = $1 AND therapist_id = $2
             ORDER BY is_default DESC, name ASC",
        )
        .bind(client_id)
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Option<ClientSessionType>, ClientError> {
        sqlx::query_as::<_, ClientSessionType>(
            "SELECT id, therapist_id, client_id, name, duration_mins, rate_inr,
                    mode, description, is_active, is_default, created_at, updated_at
             FROM client_session_types
             WHERE id = $1 AND client_id = $2 AND therapist_id = $3",
        )
        .bind(id)
        .bind(client_id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        input: &CreateClientSessionTypeInput,
    ) -> Result<ClientSessionType, ClientError> {
        let is_active = input.is_active.unwrap_or(true);
        let is_default = input.is_default.unwrap_or(false);

        sqlx::query_as::<_, ClientSessionType>(
            "INSERT INTO client_session_types
                (therapist_id, client_id, name, duration_mins, rate_inr,
                 mode, description, is_active, is_default)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, therapist_id, client_id, name, duration_mins, rate_inr,
                       mode, description, is_active, is_default, created_at, updated_at",
        )
        .bind(therapist_id)
        .bind(client_id)
        .bind(&input.name)
        .bind(input.duration_mins)
        .bind(input.rate_inr)
        .bind(&input.mode)
        .bind(&input.description)
        .bind(is_active)
        .bind(is_default)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
        input: &UpdateClientSessionTypeInput,
    ) -> Result<ClientSessionType, ClientError> {
        sqlx::query_as::<_, ClientSessionType>(
            "UPDATE client_session_types SET
                name        = COALESCE($4, name),
                duration_mins = COALESCE($5, duration_mins),
                rate_inr    = COALESCE($6, rate_inr),
                mode        = COALESCE($7, mode),
                description = COALESCE($8, description),
                is_active   = COALESCE($9, is_active),
                is_default  = COALESCE($10, is_default),
                updated_at  = now()
             WHERE id = $1 AND client_id = $2 AND therapist_id = $3
             RETURNING id, therapist_id, client_id, name, duration_mins, rate_inr,
                       mode, description, is_active, is_default, created_at, updated_at",
        )
        .bind(id)
        .bind(client_id)
        .bind(therapist_id)
        .bind(&input.name)
        .bind(input.duration_mins)
        .bind(input.rate_inr)
        .bind(&input.mode)
        .bind(&input.description)
        .bind(input.is_active)
        .bind(input.is_default)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn delete(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), ClientError> {
        let result = sqlx::query(
            "DELETE FROM client_session_types
             WHERE id = $1 AND client_id = $2 AND therapist_id = $3",
        )
        .bind(id)
        .bind(client_id)
        .bind(therapist_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ClientError::ClientNotFound);
        }
        Ok(())
    }

    async fn set_default(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), ClientError> {
        // Verify the target row exists and belongs to this therapist/client
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM client_session_types
                WHERE id = $1 AND client_id = $2 AND therapist_id = $3
             )",
        )
        .bind(id)
        .bind(client_id)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        if !exists {
            return Err(ClientError::ClientNotFound);
        }

        // Use a transaction: clear existing default, then set new default
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ClientError::Database(e.to_string()))?;

        sqlx::query(
            "UPDATE client_session_types
             SET is_default = false, updated_at = now()
             WHERE client_id = $1 AND therapist_id = $2",
        )
        .bind(client_id)
        .bind(therapist_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        sqlx::query(
            "UPDATE client_session_types
             SET is_default = true, updated_at = now()
             WHERE id = $1 AND client_id = $2 AND therapist_id = $3",
        )
        .bind(id)
        .bind(client_id)
        .bind(therapist_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ClientError::Database(e.to_string()))?;

        Ok(())
    }
}
