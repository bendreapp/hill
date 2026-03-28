use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::iam::domain::entity::OnboardingToken;
use crate::iam::domain::error::IamError;
use crate::iam::domain::port::OnboardingTokenRepository;

pub struct PgOnboardingTokenRepository {
    pool: PgPool,
}

impl PgOnboardingTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OnboardingTokenRepository for PgOnboardingTokenRepository {
    async fn create(
        &self,
        therapist_id: Uuid,
        label: Option<&str>,
        max_uses: Option<i32>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<OnboardingToken, IamError> {
        sqlx::query_as::<_, OnboardingToken>(
            "INSERT INTO client_onboarding_tokens (therapist_id, label, max_uses, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, therapist_id, token, label, is_active, max_uses, use_count, expires_at, created_at"
        )
        .bind(therapist_id)
        .bind(label)
        .bind(max_uses)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn find_by_token(&self, token: Uuid) -> Result<Option<OnboardingToken>, IamError> {
        sqlx::query_as::<_, OnboardingToken>(
            "SELECT id, therapist_id, token, label, is_active, max_uses, use_count, expires_at, created_at
            FROM client_onboarding_tokens
            WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn list_by_therapist(&self, therapist_id: Uuid) -> Result<Vec<OnboardingToken>, IamError> {
        sqlx::query_as::<_, OnboardingToken>(
            "SELECT id, therapist_id, token, label, is_active, max_uses, use_count, expires_at, created_at
            FROM client_onboarding_tokens
            WHERE therapist_id = $1
            ORDER BY created_at DESC"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn toggle_active(&self, id: Uuid, is_active: bool) -> Result<OnboardingToken, IamError> {
        sqlx::query_as::<_, OnboardingToken>(
            "UPDATE client_onboarding_tokens SET is_active = $2
            WHERE id = $1
            RETURNING id, therapist_id, token, label, is_active, max_uses, use_count, expires_at, created_at"
        )
        .bind(id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn increment_use_count(&self, id: Uuid) -> Result<(), IamError> {
        sqlx::query("UPDATE client_onboarding_tokens SET use_count = use_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| IamError::Database(e.to_string()))?;
        Ok(())
    }
}
