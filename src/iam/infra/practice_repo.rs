use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::iam::domain::entity::{Practice, PracticeMember};
use crate::iam::domain::error::IamError;
use crate::iam::domain::port::PracticeRepository;

pub struct PgPracticeRepository {
    pool: PgPool,
}

impl PgPracticeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PracticeRepository for PgPracticeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Practice>, IamError> {
        sqlx::query_as::<_, Practice>(
            "SELECT id, name, owner_id, created_at, updated_at FROM practices WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Option<Practice>, IamError> {
        sqlx::query_as::<_, Practice>(
            "SELECT id, name, owner_id, created_at, updated_at FROM practices WHERE owner_id = $1"
        )
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn find_by_member(&self, user_id: Uuid) -> Result<Option<(Practice, PracticeMember)>, IamError> {
        let row = sqlx::query(
            "SELECT
                p.id as p_id, p.name as p_name, p.owner_id as p_owner_id,
                p.created_at as p_created_at, p.updated_at as p_updated_at,
                pm.id as pm_id, pm.practice_id as pm_practice_id, pm.user_id as pm_user_id,
                pm.therapist_id as pm_therapist_id, pm.role::text as pm_role,
                pm.can_view_notes as pm_can_view_notes,
                pm.created_at as pm_created_at, pm.updated_at as pm_updated_at
            FROM practice_members pm
            JOIN practices p ON p.id = pm.practice_id
            WHERE pm.user_id = $1
            LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))?;

        Ok(row.map(|r| {
            (
                Practice {
                    id: r.get("p_id"),
                    name: r.get("p_name"),
                    owner_id: r.get("p_owner_id"),
                    created_at: r.get("p_created_at"),
                    updated_at: r.get("p_updated_at"),
                },
                PracticeMember {
                    id: r.get("pm_id"),
                    practice_id: r.get("pm_practice_id"),
                    user_id: r.get("pm_user_id"),
                    therapist_id: r.get("pm_therapist_id"),
                    role: r.get("pm_role"),
                    can_view_notes: r.get("pm_can_view_notes"),
                    created_at: r.get("pm_created_at"),
                    updated_at: r.get("pm_updated_at"),
                },
            )
        }))
    }

    async fn create(&self, name: &str, owner_id: Uuid) -> Result<Practice, IamError> {
        sqlx::query_as::<_, Practice>(
            "INSERT INTO practices (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at, updated_at"
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn list_members(&self, practice_id: Uuid) -> Result<Vec<PracticeMember>, IamError> {
        sqlx::query_as::<_, PracticeMember>(
            "SELECT id, practice_id, user_id, therapist_id, role::text as role,
                   can_view_notes, created_at, updated_at
            FROM practice_members
            WHERE practice_id = $1
            ORDER BY created_at"
        )
        .bind(practice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn find_member(&self, practice_id: Uuid, user_id: Uuid) -> Result<Option<PracticeMember>, IamError> {
        sqlx::query_as::<_, PracticeMember>(
            "SELECT id, practice_id, user_id, therapist_id, role::text as role,
                   can_view_notes, created_at, updated_at
            FROM practice_members
            WHERE practice_id = $1 AND user_id = $2"
        )
        .bind(practice_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn update_member(&self, member_id: Uuid, role: &str, can_view_notes: bool) -> Result<PracticeMember, IamError> {
        sqlx::query_as::<_, PracticeMember>(
            "UPDATE practice_members
            SET role = $2::practice_role, can_view_notes = $3, updated_at = now()
            WHERE id = $1
            RETURNING id, practice_id, user_id, therapist_id, role::text as role,
                      can_view_notes, created_at, updated_at"
        )
        .bind(member_id)
        .bind(role)
        .bind(can_view_notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn remove_member(&self, member_id: Uuid) -> Result<(), IamError> {
        sqlx::query("DELETE FROM practice_members WHERE id = $1")
            .bind(member_id)
            .execute(&self.pool)
            .await
            .map_err(|e| IamError::Database(e.to_string()))?;
        Ok(())
    }

    async fn add_member(
        &self,
        practice_id: Uuid,
        user_id: Uuid,
        therapist_id: Option<Uuid>,
        role: &str,
        can_view_notes: bool,
    ) -> Result<PracticeMember, IamError> {
        sqlx::query_as::<_, PracticeMember>(
            "INSERT INTO practice_members (practice_id, user_id, therapist_id, role, can_view_notes)
            VALUES ($1, $2, $3, $4::practice_role, $5)
            ON CONFLICT (practice_id, user_id) DO UPDATE SET role = $4::practice_role, can_view_notes = $5
            RETURNING id, practice_id, user_id, therapist_id, role::text as role,
                      can_view_notes, created_at, updated_at"
        )
        .bind(practice_id)
        .bind(user_id)
        .bind(therapist_id)
        .bind(role)
        .bind(can_view_notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn get_accessible_therapist_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, IamError> {
        let rows = sqlx::query_scalar::<_, Uuid>(
            "SELECT DISTINCT pm2.user_id
            FROM practice_members pm1
            JOIN practice_members pm2 ON pm1.practice_id = pm2.practice_id
            WHERE pm1.user_id = $1
              AND pm1.role::text IN ('owner', 'admin')"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))?;

        Ok(rows)
    }
}
