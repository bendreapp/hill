use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engagement::domain::entity::{
    CreateIntakeFormInput, CreateIntakeResponseInput, IntakeForm, IntakeResponse,
    UpdateIntakeFormInput,
};
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::IntakeFormRepository;

pub struct PgIntakeFormRepository {
    pool: PgPool,
}

impl PgIntakeFormRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IntakeFormRepository for PgIntakeFormRepository {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<IntakeForm>, EngagementError> {
        sqlx::query_as::<_, IntakeForm>(
            "SELECT
                id, therapist_id, name, description,
                form_type,
                status::text as status,
                fields,
                created_at, updated_at
            FROM intake_forms
            WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IntakeForm>, i64), EngagementError> {
        let rows = sqlx::query_as::<_, IntakeForm>(
            "SELECT
                id, therapist_id, name, description,
                form_type,
                status::text as status,
                fields,
                created_at, updated_at
            FROM intake_forms
            WHERE therapist_id = ANY($1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3"
        )
        .bind(therapist_ids)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM intake_forms
            WHERE therapist_id = ANY($1)"
        )
        .bind(therapist_ids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeFormInput,
    ) -> Result<IntakeForm, EngagementError> {
        let form_type = input.form_type.as_deref().unwrap_or("individual");
        let status = input.status.as_deref().unwrap_or("draft");
        let fields = input
            .fields
            .clone()
            .unwrap_or_else(|| serde_json::json!([]));

        sqlx::query_as::<_, IntakeForm>(
            "INSERT INTO intake_forms (therapist_id, name, description, form_type, status, fields)
            VALUES ($1, $2, $3, $4, $5::intake_form_status, $6)
            RETURNING
                id, therapist_id, name, description,
                form_type,
                status::text as status,
                fields,
                created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(form_type)
        .bind(status)
        .bind(&fields)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateIntakeFormInput,
    ) -> Result<IntakeForm, EngagementError> {
        sqlx::query_as::<_, IntakeForm>(
            "UPDATE intake_forms SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                form_type = COALESCE($4, form_type),
                status = COALESCE($5::intake_form_status, status),
                fields = COALESCE($6, fields),
                updated_at = now()
            WHERE id = $1 AND therapist_id = $7
            RETURNING
                id, therapist_id, name, description,
                form_type,
                status::text as status,
                fields,
                created_at, updated_at"
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.form_type)
        .bind(&input.status)
        .bind(&input.fields)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), EngagementError> {
        sqlx::query("DELETE FROM intake_forms WHERE id = $1 AND therapist_id = $2")
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(())
    }

    async fn create_response(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeResponseInput,
        form_snapshot: &serde_json::Value,
    ) -> Result<IntakeResponse, EngagementError> {
        sqlx::query_as::<_, IntakeResponse>(
            "INSERT INTO intake_responses (
                therapist_id, client_id, intake_form_id,
                session_id, form_snapshot, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id, therapist_id, client_id, intake_form_id,
                session_id, access_token,
                status::text as status,
                responses, form_snapshot,
                submitted_at, expires_at,
                created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(&input.client_id)
        .bind(&input.intake_form_id)
        .bind(&input.session_id)
        .bind(form_snapshot)
        .bind(&input.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn find_response_by_id(
        &self,
        id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Option<IntakeResponse>, EngagementError> {
        sqlx::query_as::<_, IntakeResponse>(
            "SELECT
                id, therapist_id, client_id, intake_form_id,
                session_id, access_token,
                status::text as status,
                responses, form_snapshot,
                submitted_at, expires_at,
                created_at, updated_at
            FROM intake_responses
            WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn find_response_by_token(
        &self,
        token: Uuid,
    ) -> Result<Option<IntakeResponse>, EngagementError> {
        sqlx::query_as::<_, IntakeResponse>(
            "SELECT
                id, therapist_id, client_id, intake_form_id,
                session_id, access_token,
                status::text as status,
                responses, form_snapshot,
                submitted_at, expires_at,
                created_at, updated_at
            FROM intake_responses
            WHERE access_token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn list_responses_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IntakeResponse>, i64), EngagementError> {
        let rows = sqlx::query_as::<_, IntakeResponse>(
            "SELECT
                id, therapist_id, client_id, intake_form_id,
                session_id, access_token,
                status::text as status,
                responses, form_snapshot,
                submitted_at, expires_at,
                created_at, updated_at
            FROM intake_responses
            WHERE client_id = $1 AND therapist_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4"
        )
        .bind(client_id)
        .bind(therapist_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM intake_responses WHERE client_id = $1 AND therapist_id = $2"
        )
        .bind(client_id)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn submit_response(
        &self,
        id: Uuid,
        encrypted_responses: &str,
    ) -> Result<IntakeResponse, EngagementError> {
        sqlx::query_as::<_, IntakeResponse>(
            "UPDATE intake_responses SET
                responses = $2,
                status = 'submitted'::intake_response_status,
                submitted_at = now(),
                updated_at = now()
            WHERE id = $1
            RETURNING
                id, therapist_id, client_id, intake_form_id,
                session_id, access_token,
                status::text as status,
                responses, form_snapshot,
                submitted_at, expires_at,
                created_at, updated_at"
        )
        .bind(id)
        .bind(encrypted_responses)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }
}
