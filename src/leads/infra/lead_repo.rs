use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::leads::domain::entity::*;
use crate::leads::domain::error::LeadsError;
use crate::leads::domain::port::{LeadRepository, TherapistInfo};

pub struct PgLeadRepository {
    pool: PgPool,
}

impl PgLeadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeadRepository for PgLeadRepository {
    async fn create(&self, therapist_id: Uuid, input: &CreateLeadInput) -> Result<Lead, LeadsError> {
        let preferred_times_json: Option<serde_json::Value> = input.preferred_times
            .as_ref()
            .map(|times| serde_json::to_value(times).unwrap_or(serde_json::Value::Null));

        let row = sqlx::query_as::<_, Lead>(
            "INSERT INTO leads (therapist_id, full_name, email, phone, reason, source, status, preferred_times, message)
             VALUES ($1, $2, $3, $4, $5, $6, 'new', $7, $8)
             RETURNING id, therapist_id, full_name, email, phone, reason, source, status::text as status,
                       session_id, client_id, converted_client_id, notes, preferred_times, message, created_at, updated_at"
        )
        .bind(therapist_id)
        .bind(&input.full_name)
        .bind(&input.email)
        .bind(&input.phone)
        .bind(&input.reason)
        .bind(input.source.as_deref().unwrap_or("booking"))
        .bind(preferred_times_json)
        .bind(&input.message)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<Lead>, LeadsError> {
        let row = sqlx::query_as::<_, Lead>(
            "SELECT id, therapist_id, full_name, email, phone, reason, source, status::text as status,
                    session_id, client_id, converted_client_id, notes, preferred_times, message, created_at, updated_at
             FROM leads WHERE id = $1 AND therapist_id = $2"
        )
        .bind(id)
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn list_by_therapist(
        &self,
        therapist_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Lead>, i64), LeadsError> {
        let (rows, total) = if let Some(s) = status {
            let rows = sqlx::query_as::<_, Lead>(
                "SELECT id, therapist_id, full_name, email, phone, reason, source, status::text as status,
                        session_id, client_id, converted_client_id, notes, preferred_times, message, created_at, updated_at
                 FROM leads WHERE therapist_id = $1 AND status = $2::lead_status
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(therapist_id)
            .bind(s)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)::bigint FROM leads WHERE therapist_id = $1 AND status = $2::lead_status"
            )
            .bind(therapist_id)
            .bind(s)
            .fetch_one(&self.pool)
            .await?;

            (rows, total)
        } else {
            let rows = sqlx::query_as::<_, Lead>(
                "SELECT id, therapist_id, full_name, email, phone, reason, source, status::text as status,
                        session_id, client_id, converted_client_id, notes, preferred_times, message, created_at, updated_at
                 FROM leads WHERE therapist_id = $1
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(therapist_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)::bigint FROM leads WHERE therapist_id = $1"
            )
            .bind(therapist_id)
            .fetch_one(&self.pool)
            .await?;

            (rows, total)
        };
        Ok((rows, total))
    }

    async fn update(&self, id: Uuid, therapist_id: Uuid, input: &UpdateLeadInput) -> Result<Lead, LeadsError> {
        let row = sqlx::query_as::<_, Lead>(
            "UPDATE leads SET
                status = COALESCE($3::lead_status, status),
                notes = COALESCE($4, notes),
                client_id = COALESCE($5, client_id),
                updated_at = now()
             WHERE id = $1 AND therapist_id = $2
             RETURNING id, therapist_id, full_name, email, phone, reason, source, status::text as status,
                       session_id, client_id, converted_client_id, notes, preferred_times, message, created_at, updated_at"
        )
        .bind(id)
        .bind(therapist_id)
        .bind(&input.status)
        .bind(&input.notes)
        .bind(input.client_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_therapist_id_by_slug(&self, slug: &str) -> Result<Option<Uuid>, LeadsError> {
        let id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM therapists WHERE slug = $1 AND booking_page_active = true"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        Ok(id)
    }

    async fn mark_converted(
        &self,
        lead_id: Uuid,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Lead, LeadsError> {
        let row = sqlx::query_as::<_, Lead>(
            "UPDATE leads SET
                status = 'converted'::lead_status,
                converted_client_id = $3,
                updated_at = now()
             WHERE id = $1 AND therapist_id = $2
             RETURNING id, therapist_id, full_name, email, phone, reason, source, status::text as status,
                       session_id, client_id, converted_client_id, notes, preferred_times, message, created_at, updated_at"
        )
        .bind(lead_id)
        .bind(therapist_id)
        .bind(client_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_therapist_email(&self, therapist_id: Uuid) -> Result<Option<String>, LeadsError> {
        let email = sqlx::query_scalar::<_, String>(
            "SELECT email FROM auth.users WHERE id = $1"
        )
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(email)
    }

    async fn find_therapist_info(&self, therapist_id: Uuid) -> Result<Option<TherapistInfo>, LeadsError> {
        // Use a tuple since sqlx::FromRow can't span two tables inline
        let row = sqlx::query_as::<_, (String, Option<String>, bool, bool, Option<String>)>(
            "SELECT t.full_name, t.display_name, t.comms_email, t.comms_whatsapp,
                    u.email
             FROM therapists t
             LEFT JOIN auth.users u ON u.id = t.id
             WHERE t.id = $1"
        )
        .bind(therapist_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(full_name, display_name, comms_email, comms_whatsapp, email)| {
            TherapistInfo { full_name, display_name, email, comms_email, comms_whatsapp }
        }))
    }
}
