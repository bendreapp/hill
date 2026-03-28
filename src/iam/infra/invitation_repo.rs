use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::iam::domain::entity::PracticeInvitation;
use crate::iam::domain::error::IamError;
use crate::iam::domain::port::InvitationRepository;

pub struct PgInvitationRepository {
    pool: PgPool,
}

impl PgInvitationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvitationRepository for PgInvitationRepository {
    async fn create(
        &self,
        practice_id: Uuid,
        invited_by: Uuid,
        email: Option<&str>,
        role: &str,
        can_view_notes: bool,
    ) -> Result<PracticeInvitation, IamError> {
        sqlx::query_as::<_, PracticeInvitation>(
            "INSERT INTO practice_invitations (practice_id, invited_by, email, role, can_view_notes)
            VALUES ($1, $2, $3, $4::practice_role, $5)
            RETURNING id, practice_id, invited_by, token, email,
                      role::text as role, can_view_notes, status::text as status,
                      accepted_by, accepted_at, expires_at, created_at"
        )
        .bind(practice_id)
        .bind(invited_by)
        .bind(email)
        .bind(role)
        .bind(can_view_notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn find_by_token(&self, token: Uuid) -> Result<Option<PracticeInvitation>, IamError> {
        sqlx::query_as::<_, PracticeInvitation>(
            "SELECT id, practice_id, invited_by, token, email,
                   role::text as role, can_view_notes, status::text as status,
                   accepted_by, accepted_at, expires_at, created_at
            FROM practice_invitations
            WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn list_by_practice(&self, practice_id: Uuid) -> Result<Vec<PracticeInvitation>, IamError> {
        sqlx::query_as::<_, PracticeInvitation>(
            "SELECT id, practice_id, invited_by, token, email,
                   role::text as role, can_view_notes, status::text as status,
                   accepted_by, accepted_at, expires_at, created_at
            FROM practice_invitations
            WHERE practice_id = $1
            ORDER BY created_at DESC"
        )
        .bind(practice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn accept(&self, id: Uuid, accepted_by: Uuid) -> Result<PracticeInvitation, IamError> {
        sqlx::query_as::<_, PracticeInvitation>(
            "UPDATE practice_invitations
            SET status = 'accepted', accepted_by = $2, accepted_at = now()
            WHERE id = $1
            RETURNING id, practice_id, invited_by, token, email,
                      role::text as role, can_view_notes, status::text as status,
                      accepted_by, accepted_at, expires_at, created_at"
        )
        .bind(id)
        .bind(accepted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn revoke(&self, id: Uuid) -> Result<(), IamError> {
        sqlx::query("UPDATE practice_invitations SET status = 'revoked' WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| IamError::Database(e.to_string()))?;
        Ok(())
    }
}
