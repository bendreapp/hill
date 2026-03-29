use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clinical::domain::entity::{CreateNoteInput, SessionNote, UpdateNoteInput};
use crate::clinical::domain::error::ClinicalError;
use crate::clinical::domain::port::NoteRepository;

pub struct PgNoteRepository {
    pool: PgPool,
}

impl PgNoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NoteRepository for PgNoteRepository {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<SessionNote>, ClinicalError> {
        sqlx::query_as::<_, SessionNote>(
            "SELECT
                id, session_id, therapist_id,
                note_type::text as note_type,
                subjective, objective, assessment, plan,
                freeform_content, homework,
                techniques_used, risk_flags,
                deleted_at, created_at, updated_at
            FROM session_notes
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn find_by_session(&self, session_id: Uuid, therapist_id: Uuid) -> Result<Option<SessionNote>, ClinicalError> {
        sqlx::query_as::<_, SessionNote>(
            "SELECT
                id, session_id, therapist_id,
                note_type::text as note_type,
                subjective, objective, assessment, plan,
                freeform_content, homework,
                techniques_used, risk_flags,
                deleted_at, created_at, updated_at
            FROM session_notes
            WHERE session_id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(session_id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn list_by_therapist(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SessionNote>, i64), ClinicalError> {
        let rows = sqlx::query_as::<_, SessionNote>(
            "SELECT
                id, session_id, therapist_id,
                note_type::text as note_type,
                subjective, objective, assessment, plan,
                freeform_content, homework,
                techniques_used, risk_flags,
                deleted_at, created_at, updated_at
            FROM session_notes
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
            FROM session_notes
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
        input: &CreateNoteInput,
    ) -> Result<SessionNote, ClinicalError> {
        let note_type = input.note_type.as_deref().unwrap_or("freeform");

        sqlx::query_as::<_, SessionNote>(
            "INSERT INTO session_notes (
                session_id, therapist_id, note_type,
                subjective, objective, assessment, plan,
                freeform_content, homework,
                techniques_used, risk_flags
            )
            VALUES ($1, $2, $3::note_type, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                id, session_id, therapist_id,
                note_type::text as note_type,
                subjective, objective, assessment, plan,
                freeform_content, homework,
                techniques_used, risk_flags,
                deleted_at, created_at, updated_at"
        )
        .bind(&input.session_id)
        .bind(therapist_id)
        .bind(note_type)
        .bind(&input.subjective)
        .bind(&input.objective)
        .bind(&input.assessment)
        .bind(&input.plan)
        .bind(&input.freeform_content)
        .bind(&input.homework)
        .bind(&input.techniques_used)
        .bind(&input.risk_flags)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateNoteInput,
    ) -> Result<SessionNote, ClinicalError> {
        sqlx::query_as::<_, SessionNote>(
            "UPDATE session_notes SET
                note_type = COALESCE($2::note_type, note_type),
                subjective = COALESCE($3, subjective),
                objective = COALESCE($4, objective),
                assessment = COALESCE($5, assessment),
                plan = COALESCE($6, plan),
                freeform_content = COALESCE($7, freeform_content),
                homework = COALESCE($8, homework),
                techniques_used = COALESCE($9, techniques_used),
                risk_flags = COALESCE($10, risk_flags),
                updated_at = now()
            WHERE id = $1 AND therapist_id = $11 AND deleted_at IS NULL
            RETURNING
                id, session_id, therapist_id,
                note_type::text as note_type,
                subjective, objective, assessment, plan,
                freeform_content, homework,
                techniques_used, risk_flags,
                deleted_at, created_at, updated_at"
        )
        .bind(id)
        .bind(&input.note_type)
        .bind(&input.subjective)
        .bind(&input.objective)
        .bind(&input.assessment)
        .bind(&input.plan)
        .bind(&input.freeform_content)
        .bind(&input.homework)
        .bind(&input.techniques_used)
        .bind(&input.risk_flags)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClinicalError::Database(e.to_string()))
    }

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClinicalError> {
        sqlx::query("UPDATE session_notes SET deleted_at = now() WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ClinicalError::Database(e.to_string()))?;

        Ok(())
    }
}
