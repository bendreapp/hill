use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clinical::domain::entity::{
    CreateTreatmentPlanInput, TreatmentPlan, UpdateTreatmentPlanInput,
};
use crate::clinical::domain::error::ClinicalError;
use crate::clinical::domain::port::TreatmentPlanRepository;

pub struct PgTreatmentPlanRepository {
    pool: PgPool,
}

impl PgTreatmentPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TreatmentPlanRepository for PgTreatmentPlanRepository {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<TreatmentPlan>, ClinicalError> {
        sqlx::query_as::<_, TreatmentPlan>(
            "SELECT
                id, therapist_id, client_id, title,
                modality::text as modality,
                modality_other, presenting_concerns, diagnosis,
                goals,
                status::text as status,
                start_date, target_end_date, notes,
                deleted_at, created_at, updated_at
            FROM treatment_plans
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn list_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TreatmentPlan>, i64), ClinicalError> {
        let rows = sqlx::query_as::<_, TreatmentPlan>(
            "SELECT
                id, therapist_id, client_id, title,
                modality::text as modality,
                modality_other, presenting_concerns, diagnosis,
                goals,
                status::text as status,
                start_date, target_end_date, notes,
                deleted_at, created_at, updated_at
            FROM treatment_plans
            WHERE client_id = $1 AND therapist_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4"
        )
        .bind(client_id)
        .bind(therapist_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM treatment_plans
            WHERE client_id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(client_id)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn list_by_therapist(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TreatmentPlan>, i64), ClinicalError> {
        let rows = sqlx::query_as::<_, TreatmentPlan>(
            "SELECT
                id, therapist_id, client_id, title,
                modality::text as modality,
                modality_other, presenting_concerns, diagnosis,
                goals,
                status::text as status,
                start_date, target_end_date, notes,
                deleted_at, created_at, updated_at
            FROM treatment_plans
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3"
        )
        .bind(therapist_ids)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM treatment_plans
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL"
        )
        .bind(therapist_ids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateTreatmentPlanInput,
    ) -> Result<TreatmentPlan, ClinicalError> {
        let title = input.title.as_deref().unwrap_or("Treatment Plan");
        let modality = input.modality.as_deref().unwrap_or("cbt");
        let status = input.status.as_deref().unwrap_or("draft");

        sqlx::query_as::<_, TreatmentPlan>(
            "INSERT INTO treatment_plans (
                therapist_id, client_id, title, modality,
                modality_other, presenting_concerns, diagnosis,
                goals, status, start_date, target_end_date, notes
            )
            VALUES (
                $1, $2, $3, $4::therapy_modality,
                $5, $6, $7,
                $8, $9::treatment_plan_status, $10, $11, $12
            )
            RETURNING
                id, therapist_id, client_id, title,
                modality::text as modality,
                modality_other, presenting_concerns, diagnosis,
                goals,
                status::text as status,
                start_date, target_end_date, notes,
                deleted_at, created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(&input.client_id)
        .bind(title)
        .bind(modality)
        .bind(&input.modality_other)
        .bind(&input.presenting_concerns)
        .bind(&input.diagnosis)
        .bind(&input.goals)
        .bind(status)
        .bind(&input.start_date)
        .bind(&input.target_end_date)
        .bind(&input.notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateTreatmentPlanInput,
    ) -> Result<TreatmentPlan, ClinicalError> {
        sqlx::query_as::<_, TreatmentPlan>(
            "UPDATE treatment_plans SET
                title = COALESCE($2, title),
                modality = COALESCE($3::therapy_modality, modality),
                modality_other = COALESCE($4, modality_other),
                presenting_concerns = COALESCE($5, presenting_concerns),
                diagnosis = COALESCE($6, diagnosis),
                goals = COALESCE($7, goals),
                status = COALESCE($8::treatment_plan_status, status),
                start_date = COALESCE($9, start_date),
                target_end_date = COALESCE($10, target_end_date),
                notes = COALESCE($11, notes),
                updated_at = now()
            WHERE id = $1 AND therapist_id = $12 AND deleted_at IS NULL
            RETURNING
                id, therapist_id, client_id, title,
                modality::text as modality,
                modality_other, presenting_concerns, diagnosis,
                goals,
                status::text as status,
                start_date, target_end_date, notes,
                deleted_at, created_at, updated_at"
        )
        .bind(id)
        .bind(&input.title)
        .bind(&input.modality)
        .bind(&input.modality_other)
        .bind(&input.presenting_concerns)
        .bind(&input.diagnosis)
        .bind(&input.goals)
        .bind(&input.status)
        .bind(&input.start_date)
        .bind(&input.target_end_date)
        .bind(&input.notes)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClinicalError> {
        sqlx::query("UPDATE treatment_plans SET deleted_at = now() WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok(())
    }
}
