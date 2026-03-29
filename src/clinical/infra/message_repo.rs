use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clinical::domain::entity::{CreateMessageInput, Message};
use crate::clinical::domain::error::ClinicalError;
use crate::clinical::domain::port::MessageRepository;

pub struct PgMessageRepository {
    pool: PgPool,
}

impl PgMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for PgMessageRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, ClinicalError> {
        sqlx::query_as::<_, Message>(
            "SELECT
                id, therapist_id, client_id,
                sender_type::text as sender_type,
                content,
                read_at, deleted_at, created_at
            FROM messages
            WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn list_thread(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64), ClinicalError> {
        let rows = sqlx::query_as::<_, Message>(
            "SELECT
                id, therapist_id, client_id,
                sender_type::text as sender_type,
                content,
                read_at, deleted_at, created_at
            FROM messages
            WHERE therapist_id = $1 AND client_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4"
        )
        .bind(therapist_id)
        .bind(client_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM messages
            WHERE therapist_id = $1 AND client_id = $2 AND deleted_at IS NULL"
        )
        .bind(therapist_id)
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn list_unread_count(&self, therapist_id: Uuid) -> Result<i64, ClinicalError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM messages
            WHERE therapist_id = $1
              AND sender_type::text = 'client'
              AND read_at IS NULL
              AND deleted_at IS NULL"
        )
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok(count)
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateMessageInput,
    ) -> Result<Message, ClinicalError> {
        sqlx::query_as::<_, Message>(
            "INSERT INTO messages (therapist_id, client_id, sender_type, content)
            VALUES ($1, $2, $3::sender_type, $4)
            RETURNING
                id, therapist_id, client_id,
                sender_type::text as sender_type,
                content,
                read_at, deleted_at, created_at"
        )
        .bind(therapist_id)
        .bind(&input.client_id)
        .bind(&input.sender_type)
        .bind(&input.content)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn mark_read(&self, therapist_id: Uuid, message_ids: &[Uuid]) -> Result<(), ClinicalError> {
        sqlx::query(
            "UPDATE messages SET read_at = now()
            WHERE id = ANY($1) AND therapist_id = $2 AND read_at IS NULL AND deleted_at IS NULL"
        )
        .bind(message_ids)
        .bind(therapist_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok(())
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), ClinicalError> {
        sqlx::query("UPDATE messages SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok(())
    }
}
