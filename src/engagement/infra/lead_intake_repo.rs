use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engagement::domain::entity::LeadIntakeSubmission;
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::LeadIntakeSubmissionRepository;

pub struct PgLeadIntakeSubmissionRepository {
    pool: PgPool,
}

impl PgLeadIntakeSubmissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeadIntakeSubmissionRepository for PgLeadIntakeSubmissionRepository {
    async fn create(
        &self,
        lead_id: Uuid,
        therapist_id: Uuid,
        access_token: &str,
    ) -> Result<LeadIntakeSubmission, EngagementError> {
        let row = sqlx::query_as::<_, LeadIntakeSubmission>(
            "INSERT INTO lead_intake_submissions (lead_id, therapist_id, access_token)
             VALUES ($1, $2, $3)
             RETURNING id, lead_id, therapist_id, access_token, responses,
                       sent_at, submitted_at, created_at"
        )
        .bind(lead_id)
        .bind(therapist_id)
        .bind(access_token)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(row)
    }

    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<LeadIntakeSubmission>, EngagementError> {
        let row = sqlx::query_as::<_, LeadIntakeSubmission>(
            "SELECT id, lead_id, therapist_id, access_token, responses,
                    sent_at, submitted_at, created_at
             FROM lead_intake_submissions
             WHERE access_token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(row)
    }

    async fn list_by_lead(
        &self,
        lead_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Vec<LeadIntakeSubmission>, EngagementError> {
        let rows = sqlx::query_as::<_, LeadIntakeSubmission>(
            "SELECT id, lead_id, therapist_id, access_token, responses,
                    sent_at, submitted_at, created_at
             FROM lead_intake_submissions
             WHERE lead_id = $1 AND therapist_id = $2
             ORDER BY created_at DESC"
        )
        .bind(lead_id)
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn submit(
        &self,
        token: &str,
        responses: &serde_json::Value,
    ) -> Result<LeadIntakeSubmission, EngagementError> {
        let row = sqlx::query_as::<_, LeadIntakeSubmission>(
            "UPDATE lead_intake_submissions
             SET responses = $2, submitted_at = NOW()
             WHERE access_token = $1 AND submitted_at IS NULL
             RETURNING id, lead_id, therapist_id, access_token, responses,
                       sent_at, submitted_at, created_at"
        )
        .bind(token)
        .bind(responses)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(row)
    }
}
