use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::iam::domain::entity::Therapist;
use crate::iam::domain::error::IamError;
use crate::iam::domain::port::TherapistRepository;

pub struct PgTherapistRepository {
    pool: PgPool,
}

impl PgTherapistRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TherapistRepository for PgTherapistRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Therapist>, IamError> {
        sqlx::query_as::<_, Therapist>(
            "SELECT
                id, slug, full_name, display_name, bio, qualifications,
                phone, avatar_url, timezone,
                session_duration_mins, buffer_mins, session_rate_inr,
                booking_page_active, show_pricing, gstin,
                google_connected, zoom_connected,
                cancellation_hours, min_booking_advance_hours,
                no_show_charge_percent, late_cancel_charge_percent,
                cancellation_policy, late_policy, rescheduling_policy,
                custom_tags, practice_id,
                whatsapp_number, team_size,
                comms_whatsapp, comms_email, comms_sms,
                plan_selected, plan_status,
                support_requested, onboarding_complete, avatar_key,
                created_at, updated_at
            FROM therapists
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Therapist>, IamError> {
        sqlx::query_as::<_, Therapist>(
            "SELECT
                id, slug, full_name, display_name, bio, qualifications,
                phone, avatar_url, timezone,
                session_duration_mins, buffer_mins, session_rate_inr,
                booking_page_active, show_pricing, gstin,
                google_connected, zoom_connected,
                cancellation_hours, min_booking_advance_hours,
                no_show_charge_percent, late_cancel_charge_percent,
                cancellation_policy, late_policy, rescheduling_policy,
                custom_tags, practice_id,
                whatsapp_number, team_size,
                comms_whatsapp, comms_email, comms_sms,
                plan_selected, plan_status,
                support_requested, onboarding_complete, avatar_key,
                created_at, updated_at
            FROM therapists
            WHERE slug = $1 AND booking_page_active = true"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn update(&self, t: &Therapist) -> Result<Therapist, IamError> {
        sqlx::query_as::<_, Therapist>(
            "UPDATE therapists SET
                slug = $2, full_name = $3, display_name = $4, bio = $5,
                qualifications = $6, phone = $7, avatar_url = $8, timezone = $9,
                session_duration_mins = $10, buffer_mins = $11, session_rate_inr = $12,
                booking_page_active = $13, show_pricing = $14, gstin = $15,
                cancellation_hours = $16, min_booking_advance_hours = $17,
                no_show_charge_percent = $18, late_cancel_charge_percent = $19,
                cancellation_policy = $20, late_policy = $21, rescheduling_policy = $22,
                custom_tags = $23, zoom_connected = $24, google_connected = $25,
                whatsapp_number = $26, team_size = $27,
                comms_whatsapp = $28, comms_email = $29, comms_sms = $30,
                plan_selected = $31, plan_status = $32,
                support_requested = $33, onboarding_complete = $34, avatar_key = $35,
                updated_at = now()
            WHERE id = $1
            RETURNING
                id, slug, full_name, display_name, bio, qualifications,
                phone, avatar_url, timezone,
                session_duration_mins, buffer_mins, session_rate_inr,
                booking_page_active, show_pricing, gstin,
                google_connected, zoom_connected,
                cancellation_hours, min_booking_advance_hours,
                no_show_charge_percent, late_cancel_charge_percent,
                cancellation_policy, late_policy, rescheduling_policy,
                custom_tags, practice_id,
                whatsapp_number, team_size,
                comms_whatsapp, comms_email, comms_sms,
                plan_selected, plan_status,
                support_requested, onboarding_complete, avatar_key,
                created_at, updated_at"
        )
        .bind(t.id)
        .bind(&t.slug)
        .bind(&t.full_name)
        .bind(&t.display_name)
        .bind(&t.bio)
        .bind(&t.qualifications)
        .bind(&t.phone)
        .bind(&t.avatar_url)
        .bind(&t.timezone)
        .bind(t.session_duration_mins)
        .bind(t.buffer_mins)
        .bind(t.session_rate_inr)
        .bind(t.booking_page_active)
        .bind(t.show_pricing)
        .bind(&t.gstin)
        .bind(t.cancellation_hours)
        .bind(t.min_booking_advance_hours)
        .bind(t.no_show_charge_percent)
        .bind(t.late_cancel_charge_percent)
        .bind(&t.cancellation_policy)
        .bind(&t.late_policy)
        .bind(&t.rescheduling_policy)
        .bind(&t.custom_tags)
        .bind(t.zoom_connected)
        .bind(t.google_connected)
        .bind(&t.whatsapp_number)
        .bind(t.team_size)
        .bind(t.comms_whatsapp)
        .bind(t.comms_email)
        .bind(t.comms_sms)
        .bind(&t.plan_selected)
        .bind(&t.plan_status)
        .bind(t.support_requested)
        .bind(t.onboarding_complete)
        .bind(&t.avatar_key)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn slug_exists(&self, slug: &str, exclude_id: Option<Uuid>) -> Result<bool, IamError> {
        let row = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM therapists WHERE slug = $1 AND id != $2)"
        )
        .bind(slug)
        .bind(exclude_id.unwrap_or(Uuid::nil()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))?;

        Ok(row)
    }
}
