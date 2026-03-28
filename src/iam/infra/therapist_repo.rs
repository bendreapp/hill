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
                booking_page_active, gstin,
                google_connected, zoom_connected,
                cancellation_hours, min_booking_advance_hours,
                no_show_charge_percent, late_cancel_charge_percent,
                cancellation_policy, late_policy, rescheduling_policy,
                custom_tags, practice_id,
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
                booking_page_active, gstin,
                google_connected, zoom_connected,
                cancellation_hours, min_booking_advance_hours,
                no_show_charge_percent, late_cancel_charge_percent,
                cancellation_policy, late_policy, rescheduling_policy,
                custom_tags, practice_id,
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
                booking_page_active = $13, gstin = $14,
                cancellation_hours = $15, min_booking_advance_hours = $16,
                no_show_charge_percent = $17, late_cancel_charge_percent = $18,
                cancellation_policy = $19, late_policy = $20, rescheduling_policy = $21,
                custom_tags = $22, zoom_connected = $23, google_connected = $24,
                updated_at = now()
            WHERE id = $1
            RETURNING
                id, slug, full_name, display_name, bio, qualifications,
                phone, avatar_url, timezone,
                session_duration_mins, buffer_mins, session_rate_inr,
                booking_page_active, gstin,
                google_connected, zoom_connected,
                cancellation_hours, min_booking_advance_hours,
                no_show_charge_percent, late_cancel_charge_percent,
                cancellation_policy, late_policy, rescheduling_policy,
                custom_tags, practice_id,
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
