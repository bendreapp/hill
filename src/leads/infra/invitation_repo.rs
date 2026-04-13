use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::leads::domain::entity::*;
use crate::leads::domain::error::LeadsError;
use crate::leads::domain::port::ClientInvitationRepository;

pub struct PgClientInvitationRepository {
    pool: PgPool,
}

impl PgClientInvitationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientInvitationRepository for PgClientInvitationRepository {
    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> Result<ClientInvitation, LeadsError> {
        let row = sqlx::query_as::<_, ClientInvitation>(
            "INSERT INTO client_invitations (therapist_id, client_id, email, phone)
             VALUES ($1, $2, $3, $4)
             RETURNING id, therapist_id, client_id, token, email, phone, status, expires_at, claimed_at, invite_sent_at, created_at"
        )
        .bind(therapist_id)
        .bind(client_id)
        .bind(email)
        .bind(phone)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<ClientInvitation>, LeadsError> {
        let row = sqlx::query_as::<_, ClientInvitation>(
            "SELECT id, therapist_id, client_id, token, email, phone, status, expires_at, claimed_at, invite_sent_at, created_at
             FROM client_invitations WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_detail_by_token(&self, token: &str) -> Result<Option<ClientInvitationDetail>, LeadsError> {
        let row = sqlx::query_as::<_, ClientInvitationDetail>(
            "SELECT
                ci.id,
                ci.token,
                ci.status,
                ci.expires_at,
                ci.claimed_at,
                c.full_name as client_full_name,
                c.email as client_email,
                COALESCE(t.display_name, t.full_name) as therapist_name,
                t.avatar_url as therapist_avatar_url,
                t.slug as therapist_slug
            FROM client_invitations ci
            JOIN clients c ON ci.client_id = c.id
            JOIN therapists t ON ci.therapist_id = t.id
            WHERE ci.token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_by_client(&self, client_id: Uuid) -> Result<Option<ClientInvitation>, LeadsError> {
        let row = sqlx::query_as::<_, ClientInvitation>(
            "SELECT id, therapist_id, client_id, token, email, phone, status, expires_at, claimed_at, invite_sent_at, created_at
             FROM client_invitations WHERE client_id = $1
             ORDER BY created_at DESC LIMIT 1"
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn claim(&self, token: &str) -> Result<ClientInvitation, LeadsError> {
        let row = sqlx::query_as::<_, ClientInvitation>(
            "UPDATE client_invitations SET status = 'accepted', claimed_at = now()
             WHERE token = $1
             RETURNING id, therapist_id, client_id, token, email, phone, status, expires_at, claimed_at, invite_sent_at, created_at"
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn mark_invite_sent(&self, invitation_id: Uuid) -> Result<ClientInvitation, LeadsError> {
        let row = sqlx::query_as::<_, ClientInvitation>(
            "UPDATE client_invitations SET invite_sent_at = now()
             WHERE id = $1
             RETURNING id, therapist_id, client_id, token, email, phone, status, expires_at, claimed_at, invite_sent_at, created_at"
        )
        .bind(invitation_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
