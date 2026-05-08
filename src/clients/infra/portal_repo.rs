use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::domain::entity::{ClientPortalProfile, PortalInvoice, PortalSession, UpdatePortalProfileInput};
use crate::clients::domain::error::ClientError;
use crate::clients::domain::port::{ClientPortalRepository, ReminderSession};

pub struct PgClientPortalRepository {
    pool: PgPool,
}

impl PgClientPortalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientPortalRepository for PgClientPortalRepository {
    async fn list_profiles_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ClientPortalProfile>, ClientError> {
        sqlx::query_as::<_, ClientPortalProfile>(
            "SELECT
                c.id,
                c.therapist_id,
                c.full_name,
                c.email,
                c.phone,
                c.emergency_contact_name,
                c.emergency_contact_phone,
                c.intake_completed,
                c.status::text as status,
                COALESCE(t.display_name, t.full_name) as therapist_name,
                t.avatar_url as therapist_avatar_url
            FROM clients c
            JOIN therapists t ON c.therapist_id = t.id
            WHERE c.user_id = $1 AND c.deleted_at IS NULL
            ORDER BY c.created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn update_profile(
        &self,
        client_id: Uuid,
        user_id: Uuid,
        input: &UpdatePortalProfileInput,
    ) -> Result<ClientPortalProfile, ClientError> {
        // Perform the update — verifies user_id ownership via WHERE clause
        let updated = sqlx::query_scalar::<_, Uuid>(
            "UPDATE clients SET
                phone = COALESCE($3, phone),
                emergency_contact_name = COALESCE($4, emergency_contact_name),
                emergency_contact_phone = COALESCE($5, emergency_contact_phone),
                updated_at = now()
            WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
            RETURNING id"
        )
        .bind(client_id)
        .bind(user_id)
        .bind(&input.phone)
        .bind(&input.emergency_contact_name)
        .bind(&input.emergency_contact_phone)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        if updated.is_none() {
            return Err(ClientError::ClientNotFound);
        }

        // Return the full profile
        sqlx::query_as::<_, ClientPortalProfile>(
            "SELECT
                c.id,
                c.therapist_id,
                c.full_name,
                c.email,
                c.phone,
                c.emergency_contact_name,
                c.emergency_contact_phone,
                c.intake_completed,
                c.status::text as status,
                COALESCE(t.display_name, t.full_name) as therapist_name,
                t.avatar_url as therapist_avatar_url
            FROM clients c
            JOIN therapists t ON c.therapist_id = t.id
            WHERE c.id = $1 AND c.deleted_at IS NULL"
        )
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn upcoming_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PortalSession>, ClientError> {
        sqlx::query_as::<_, PortalSession>(
            "SELECT
                s.id,
                s.starts_at,
                s.ends_at,
                s.duration_mins,
                s.status::text as status,
                s.session_type_name,
                COALESCE(t.display_name, t.full_name) as therapist_name,
                s.amount_inr
            FROM sessions s
            JOIN therapists t ON s.therapist_id = t.id
            WHERE s.client_id = $1
              AND s.starts_at > now()
              AND s.status::text IN ('scheduled', 'reschedule_requested')
              AND s.deleted_at IS NULL
            ORDER BY s.starts_at ASC
            LIMIT $2"
        )
        .bind(client_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn past_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PortalSession>, i64), ClientError> {
        let rows = sqlx::query_as::<_, PortalSession>(
            "SELECT
                s.id,
                s.starts_at,
                s.ends_at,
                s.duration_mins,
                s.status::text as status,
                s.session_type_name,
                COALESCE(t.display_name, t.full_name) as therapist_name,
                s.amount_inr
            FROM sessions s
            JOIN therapists t ON s.therapist_id = t.id
            WHERE s.client_id = $1
              AND (s.starts_at <= now() OR s.status::text NOT IN ('scheduled', 'reschedule_requested'))
              AND s.deleted_at IS NULL
            ORDER BY s.starts_at DESC
            LIMIT $2 OFFSET $3"
        )
        .bind(client_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM sessions
            WHERE client_id = $1
              AND (starts_at <= now() OR status::text NOT IN ('scheduled', 'reschedule_requested'))
              AND deleted_at IS NULL"
        )
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn list_invoices(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<PortalInvoice>, ClientError> {
        sqlx::query_as::<_, PortalInvoice>(
            "SELECT
                id,
                invoice_number,
                amount_inr,
                gst_amount_inr,
                total_inr,
                status::text as status,
                paid_at,
                created_at
            FROM invoices
            WHERE client_id = $1
            ORDER BY created_at DESC"
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn request_reschedule(
        &self,
        session_id: Uuid,
        client_id: Uuid,
        requested_date: &str,
        requested_time: &str,
        reason: Option<&str>,
    ) -> Result<(), ClientError> {
        let rows_affected = sqlx::query(
            "UPDATE sessions SET
                status = 'reschedule_requested'::session_status,
                reschedule_requested_at = now(),
                reschedule_requested_date = $3,
                reschedule_requested_time = $4,
                reschedule_reason = $5,
                updated_at = now()
            WHERE id = $1 AND client_id = $2
              AND status IN ('scheduled'::session_status)
              AND deleted_at IS NULL"
        )
        .bind(session_id)
        .bind(client_id)
        .bind(requested_date)
        .bind(requested_time)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ClientError::ClientNotFound);
        }
        Ok(())
    }

    async fn find_session_by_id_for_client(
        &self,
        session_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<PortalSession>, ClientError> {
        sqlx::query_as::<_, PortalSession>(
            "SELECT
                s.id,
                s.starts_at,
                s.ends_at,
                s.duration_mins,
                s.status::text as status,
                s.session_type_name,
                COALESCE(t.display_name, t.full_name) as therapist_name,
                s.amount_inr
            FROM sessions s
            JOIN therapists t ON s.therapist_id = t.id
            WHERE s.id = $1 AND s.client_id = $2 AND s.deleted_at IS NULL"
        )
        .bind(session_id)
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn list_intake_forms(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, ClientError> {
        // Return pending intake responses for this client (not yet submitted)
        #[derive(sqlx::FromRow)]
        struct IntakeFormRow {
            id: Uuid,
            access_token: Uuid,
            status: String,
            expires_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
            form_name: String,
            form_description: Option<String>,
        }

        let rows = sqlx::query_as::<_, IntakeFormRow>(
            "SELECT
                ir.id,
                ir.access_token,
                ir.status::text as status,
                ir.expires_at,
                ir.created_at,
                f.name as form_name,
                f.description as form_description
            FROM intake_responses ir
            JOIN intake_forms f ON ir.intake_form_id = f.id
            WHERE ir.client_id = $1
              AND ir.status::text = 'pending'
              AND (ir.expires_at IS NULL OR ir.expires_at > now())
            ORDER BY ir.created_at DESC"
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;

        let forms: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "access_token": r.access_token,
                    "status": r.status,
                    "form_name": r.form_name,
                    "form_description": r.form_description,
                    "expires_at": r.expires_at,
                    "created_at": r.created_at,
                })
            })
            .collect();

        Ok(forms)
    }

    async fn list_sessions_for_reminder(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<ReminderSession>, ClientError> {
        sqlx::query_as::<_, ReminderSession>(
            "SELECT
                s.id,
                s.therapist_id,
                s.client_id,
                s.starts_at,
                c.email as client_email,
                c.full_name as client_name,
                COALESCE(au.email, '') as therapist_email,
                COALESCE(t.display_name, t.full_name) as therapist_name
            FROM sessions s
            JOIN clients c ON s.client_id = c.id
            JOIN therapists t ON s.therapist_id = t.id
            LEFT JOIN auth.users au ON t.id = au.id
            WHERE s.starts_at >= $1
              AND s.starts_at < $2
              AND s.status = 'scheduled'
              AND s.reminder_sent_at IS NULL
              AND s.deleted_at IS NULL
            ORDER BY s.starts_at ASC"
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))
    }

    async fn mark_reminder_sent(
        &self,
        session_id: Uuid,
    ) -> Result<(), ClientError> {
        sqlx::query(
            "UPDATE sessions SET reminder_sent_at = now(), updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;
        Ok(())
    }

    async fn clear_reschedule_request(
        &self,
        session_id: Uuid,
    ) -> Result<(), ClientError> {
        sqlx::query(
            "UPDATE sessions SET
                reschedule_requested_at = NULL,
                reschedule_requested_date = NULL,
                reschedule_requested_time = NULL,
                reschedule_reason = NULL,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?;
        Ok(())
    }

    async fn get_client_email_by_id(
        &self,
        client_id: Uuid,
    ) -> Result<Option<String>, ClientError> {
        let email = sqlx::query_scalar::<_, Option<String>>(
            "SELECT email FROM clients WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClientError::Database(e.to_string()))?
        .flatten();
        Ok(email)
    }
}
