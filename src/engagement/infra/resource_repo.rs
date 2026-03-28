use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engagement::domain::entity::{
    ClientResource, CreateResourceInput, Resource, UpdateResourceInput,
};
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::ResourceRepository;

pub struct PgResourceRepository {
    pool: PgPool,
}

impl PgResourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ResourceRepository for PgResourceRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Resource>, EngagementError> {
        sqlx::query_as::<_, Resource>(
            "SELECT
                id, therapist_id, title, description,
                resource_type::text as resource_type,
                file_url, external_url,
                modality_tags, category_tags,
                deleted_at, created_at, updated_at
            FROM resources
            WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Resource>, i64), EngagementError> {
        let rows = sqlx::query_as::<_, Resource>(
            "SELECT
                id, therapist_id, title, description,
                resource_type::text as resource_type,
                file_url, external_url,
                modality_tags, category_tags,
                deleted_at, created_at, updated_at
            FROM resources
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL
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
            FROM resources
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL"
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
        input: &CreateResourceInput,
    ) -> Result<Resource, EngagementError> {
        let resource_type = input.resource_type.as_deref().unwrap_or("file");

        sqlx::query_as::<_, Resource>(
            "INSERT INTO resources (
                therapist_id, title, description, resource_type,
                file_url, external_url, modality_tags, category_tags
            )
            VALUES ($1, $2, $3, $4::resource_type, $5, $6, $7, $8)
            RETURNING
                id, therapist_id, title, description,
                resource_type::text as resource_type,
                file_url, external_url,
                modality_tags, category_tags,
                deleted_at, created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(resource_type)
        .bind(&input.file_url)
        .bind(&input.external_url)
        .bind(input.modality_tags.as_deref())
        .bind(input.category_tags.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        input: &UpdateResourceInput,
    ) -> Result<Resource, EngagementError> {
        sqlx::query_as::<_, Resource>(
            "UPDATE resources SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                resource_type = COALESCE($4::resource_type, resource_type),
                file_url = COALESCE($5, file_url),
                external_url = COALESCE($6, external_url),
                modality_tags = COALESCE($7, modality_tags),
                category_tags = COALESCE($8, category_tags),
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING
                id, therapist_id, title, description,
                resource_type::text as resource_type,
                file_url, external_url,
                modality_tags, category_tags,
                deleted_at, created_at, updated_at"
        )
        .bind(id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.resource_type)
        .bind(&input.file_url)
        .bind(&input.external_url)
        .bind(input.modality_tags.as_deref())
        .bind(input.category_tags.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), EngagementError> {
        sqlx::query("UPDATE resources SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(())
    }

    async fn share(
        &self,
        resource_id: Uuid,
        therapist_id: Uuid,
        client_ids: &[Uuid],
        note: Option<&str>,
    ) -> Result<Vec<ClientResource>, EngagementError> {
        let mut results = Vec::with_capacity(client_ids.len());

        for client_id in client_ids {
            let cr = sqlx::query_as::<_, ClientResource>(
                "INSERT INTO client_resources (resource_id, client_id, therapist_id, note)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (resource_id, client_id) DO UPDATE SET note = COALESCE($4, client_resources.note)
                RETURNING id, resource_id, client_id, therapist_id, shared_at, note"
            )
            .bind(resource_id)
            .bind(client_id)
            .bind(therapist_id)
            .bind(note)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EngagementError::Database(e.to_string()))?;

            results.push(cr);
        }

        Ok(results)
    }

    async fn unshare(
        &self,
        resource_id: Uuid,
        client_ids: &[Uuid],
    ) -> Result<(), EngagementError> {
        sqlx::query("DELETE FROM client_resources WHERE resource_id = $1 AND client_id = ANY($2)")
            .bind(resource_id)
            .bind(client_ids)
            .execute(&self.pool)
            .await
            .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_shared_with_client(
        &self,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ClientResource>, i64), EngagementError> {
        let rows = sqlx::query_as::<_, ClientResource>(
            "SELECT id, resource_id, client_id, therapist_id, shared_at, note
            FROM client_resources
            WHERE client_id = $1
            ORDER BY shared_at DESC
            LIMIT $2 OFFSET $3"
        )
        .bind(client_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM client_resources WHERE client_id = $1"
        )
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EngagementError::Database(e.to_string()))?;

        Ok((rows, total))
    }
}
