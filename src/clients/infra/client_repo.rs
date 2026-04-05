use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::domain::entity::{Client, CreateClientInput, UpdateClientInput};
use crate::clients::domain::error::ClientError;
use crate::clients::domain::port::ClientRepository;

pub struct PgClientRepository {
    pool: PgPool,
}

impl PgClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientRepository for PgClientRepository {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<Client>, ClientError> {
        sqlx::query_as::<_, Client>(
            "SELECT
                id, therapist_id, user_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                intake_completed, is_active,
                status::text as status,
                client_type::text as client_type,
                category::text as category,
                labels,
                deleted_at, created_at, updated_at
            FROM clients
            WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Client>, i64), ClientError> {
        let rows = sqlx::query_as::<_, Client>(
            "SELECT
                id, therapist_id, user_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                intake_completed, is_active,
                status::text as status,
                client_type::text as client_type,
                category::text as category,
                labels,
                deleted_at, created_at, updated_at
            FROM clients
            WHERE therapist_id = ANY($1)
              AND deleted_at IS NULL
              AND ($2::text IS NULL OR status::text = $2)
            ORDER BY full_name ASC
            LIMIT $3 OFFSET $4"
        )
        .bind(therapist_ids)
        .bind(status_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        let total: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM clients
            WHERE therapist_id = ANY($1)
              AND deleted_at IS NULL
              AND ($2::text IS NULL OR status::text = $2)"
        )
        .bind(therapist_ids)
        .bind(status_filter)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateClientInput,
    ) -> Result<Client, ClientError> {
        let client_type = input.client_type.as_deref().unwrap_or("irregular");
        let category = input.category.as_deref().unwrap_or("indian");

        sqlx::query_as::<_, Client>(
            "INSERT INTO clients (
                therapist_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                client_type, category
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::client_type, $9::client_category)
            RETURNING
                id, therapist_id, user_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                intake_completed, is_active,
                status::text as status,
                client_type::text as client_type,
                category::text as category,
                labels,
                deleted_at, created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(&input.full_name)
        .bind(&input.email)
        .bind(&input.phone)
        .bind(&input.date_of_birth)
        .bind(&input.emergency_contact)
        .bind(&input.notes_private)
        .bind(client_type)
        .bind(category)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateClientInput,
    ) -> Result<Client, ClientError> {
        sqlx::query_as::<_, Client>(
            "UPDATE clients SET
                full_name = COALESCE($2, full_name),
                email = COALESCE($3, email),
                phone = COALESCE($4, phone),
                date_of_birth = COALESCE($5, date_of_birth),
                emergency_contact = COALESCE($6, emergency_contact),
                notes_private = COALESCE($7, notes_private),
                intake_completed = COALESCE($8, intake_completed),
                client_type = COALESCE($9::client_type, client_type),
                category = COALESCE($10::client_category, category),
                labels = COALESCE($11, labels),
                updated_at = now()
            WHERE id = $1 AND therapist_id = $12 AND deleted_at IS NULL
            RETURNING
                id, therapist_id, user_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                intake_completed, is_active,
                status::text as status,
                client_type::text as client_type,
                category::text as category,
                labels,
                deleted_at, created_at, updated_at"
        )
        .bind(id)
        .bind(&input.full_name)
        .bind(&input.email)
        .bind(&input.phone)
        .bind(&input.date_of_birth)
        .bind(&input.emergency_contact)
        .bind(&input.notes_private)
        .bind(&input.intake_completed)
        .bind(&input.client_type)
        .bind(&input.category)
        .bind(&input.labels)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClientError> {
        sqlx::query("UPDATE clients SET deleted_at = now() WHERE id = $1 AND therapist_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(therapist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ClientError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update_status(&self, id: Uuid, therapist_id: Uuid, status: &str) -> Result<Client, ClientError> {
        sqlx::query_as::<_, Client>(
            "UPDATE clients SET status = $2::client_status, updated_at = now()
            WHERE id = $1 AND therapist_id = $3 AND deleted_at IS NULL
            RETURNING
                id, therapist_id, user_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                intake_completed, is_active,
                status::text as status,
                client_type::text as client_type,
                category::text as category,
                labels,
                deleted_at, created_at, updated_at"
        )
        .bind(id)
        .bind(status)
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn find_by_email(
        &self,
        therapist_id: Uuid,
        email: &str,
    ) -> Result<Option<Client>, ClientError> {
        sqlx::query_as::<_, Client>(
            "SELECT
                id, therapist_id, user_id, full_name, email, phone,
                date_of_birth, emergency_contact, notes_private,
                intake_completed, is_active,
                status::text as status,
                client_type::text as client_type,
                category::text as category,
                labels,
                deleted_at, created_at, updated_at
            FROM clients
            WHERE therapist_id = $1 AND email = $2 AND deleted_at IS NULL"
        )
        .bind(therapist_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn count_active(&self, therapist_id: Uuid) -> Result<i64, ClientError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM clients
            WHERE therapist_id = $1
              AND deleted_at IS NULL
              AND status::text = 'active'"
        )
        .bind(therapist_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        Ok(count)
    }
}
