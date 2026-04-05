use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engagement::domain::entity::{
    CreateIntakeQuestionInput, IntakeFormQuestion, UpdateIntakeQuestionInput,
};
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::IntakeFormQuestionRepository;

pub struct PgIntakeFormQuestionRepository {
    pool: PgPool,
}

impl PgIntakeFormQuestionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLS: &str = "id, therapist_id, question_text, field_type, options, is_required, sort_order, created_at, updated_at";

#[async_trait]
impl IntakeFormQuestionRepository for PgIntakeFormQuestionRepository {
    async fn list_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<IntakeFormQuestion>, EngagementError> {
        sqlx::query_as::<_, IntakeFormQuestion>(&format!(
            "SELECT {SELECT_COLS}
             FROM intake_form_questions
             WHERE therapist_id = $1
             ORDER BY sort_order ASC, created_at ASC"
        ))
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeQuestionInput,
    ) -> Result<IntakeFormQuestion, EngagementError> {
        // Determine next sort_order if not provided
        let sort_order = match input.sort_order {
            Some(s) => s,
            None => {
                let max: Option<i32> = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(sort_order), -1) FROM intake_form_questions WHERE therapist_id = $1"
                )
                .bind(therapist_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| EngagementError::Database(e.to_string()))?;
                max.unwrap_or(-1) + 1
            }
        };

        let is_required = input.is_required.unwrap_or(false);

        sqlx::query_as::<_, IntakeFormQuestion>(&format!(
            "INSERT INTO intake_form_questions
               (therapist_id, question_text, field_type, options, is_required, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING {SELECT_COLS}"
        ))
        .bind(therapist_id)
        .bind(&input.question_text)
        .bind(&input.field_type)
        .bind(&input.options)
        .bind(is_required)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateIntakeQuestionInput,
    ) -> Result<IntakeFormQuestion, EngagementError> {
        // We need to handle the options field carefully:
        // - If is_required is None, keep existing; if Some, update
        // - Same for all other fields
        let result = sqlx::query_as::<_, IntakeFormQuestion>(&format!(
            "UPDATE intake_form_questions SET
               question_text = COALESCE($3, question_text),
               field_type    = COALESCE($4, field_type),
               options       = CASE WHEN $5::boolean THEN $6 ELSE options END,
               is_required   = COALESCE($7, is_required),
               sort_order    = COALESCE($8, sort_order),
               updated_at    = now()
             WHERE id = $1 AND therapist_id = $2
             RETURNING {SELECT_COLS}"
        ))
        .bind(id)
        .bind(therapist_id)
        .bind(&input.question_text)
        .bind(&input.field_type)
        // $5 = flag: whether to update options
        .bind(input.options.is_some())
        // $6 = new options value (may be null to clear)
        .bind(&input.options)
        .bind(input.is_required)
        .bind(input.sort_order)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        result.ok_or(EngagementError::IntakeQuestionNotFound)
    }

    async fn delete(
        &self,
        id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), EngagementError> {
        let rows = sqlx::query(
            "DELETE FROM intake_form_questions WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .execute(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        if rows.rows_affected() == 0 {
            return Err(EngagementError::IntakeQuestionNotFound);
        }
        Ok(())
    }

    async fn reorder(
        &self,
        therapist_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<IntakeFormQuestion>, EngagementError> {
        // Update each question's sort_order based on position in ids array
        for (idx, &id) in ids.iter().enumerate() {
            sqlx::query(
                "UPDATE intake_form_questions SET sort_order = $1, updated_at = now()
                 WHERE id = $2 AND therapist_id = $3"
            )
            .bind(idx as i32)
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| EngagementError::Database(e.to_string()))?;
        }

        // Return the updated list in the new order
        self.list_by_therapist(therapist_id).await
    }

    async fn seed_defaults(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<IntakeFormQuestion>, EngagementError> {
        sqlx::query("SELECT public.seed_default_intake_questions($1)")
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| EngagementError::Database(e.to_string()))?;

        self.list_by_therapist(therapist_id).await
    }
}
