use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engagement::domain::entity::MessageTemplate;
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::MessageTemplateRepository;

pub struct PgMessageTemplateRepository {
    pool: PgPool,
}

impl PgMessageTemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageTemplateRepository for PgMessageTemplateRepository {
    async fn find_all_by_therapist(&self, therapist_id: Uuid) -> Result<Vec<MessageTemplate>, EngagementError> {
        sqlx::query_as::<_, MessageTemplate>(
            "SELECT id, therapist_id, template_key, subject, body, created_at, updated_at
             FROM message_templates
             WHERE therapist_id = $1
             ORDER BY template_key ASC"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn find_by_key(&self, therapist_id: Uuid, key: &str) -> Result<Option<MessageTemplate>, EngagementError> {
        sqlx::query_as::<_, MessageTemplate>(
            "SELECT id, therapist_id, template_key, subject, body, created_at, updated_at
             FROM message_templates
             WHERE therapist_id = $1 AND template_key = $2"
        )
        .bind(therapist_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn upsert(
        &self,
        therapist_id: Uuid,
        key: &str,
        subject: &str,
        body: &str,
    ) -> Result<MessageTemplate, EngagementError> {
        sqlx::query_as::<_, MessageTemplate>(
            "INSERT INTO message_templates (therapist_id, template_key, subject, body)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (therapist_id, template_key)
             DO UPDATE SET subject = EXCLUDED.subject, body = EXCLUDED.body, updated_at = NOW()
             RETURNING id, therapist_id, template_key, subject, body, created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(key)
        .bind(subject)
        .bind(body)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }
}
