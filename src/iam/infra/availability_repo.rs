use async_trait::async_trait;
use chrono::NaiveTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::iam::domain::entity::Availability;
use crate::iam::domain::error::IamError;
use crate::iam::domain::port::AvailabilityRepository;

pub struct PgAvailabilityRepository {
    pool: PgPool,
}

impl PgAvailabilityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AvailabilityRepository for PgAvailabilityRepository {
    async fn find_by_therapist(&self, therapist_id: Uuid) -> Result<Vec<Availability>, IamError> {
        sqlx::query_as::<_, Availability>(
            "SELECT id, therapist_id, day_of_week, start_time, end_time, is_active
            FROM availability
            WHERE therapist_id = $1
            ORDER BY day_of_week, start_time"
        )
        .bind(therapist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }

    async fn upsert(
        &self,
        therapist_id: Uuid,
        day_of_week: i16,
        start_time: NaiveTime,
        end_time: NaiveTime,
        is_active: bool,
    ) -> Result<Availability, IamError> {
        sqlx::query_as::<_, Availability>(
            "INSERT INTO availability (therapist_id, day_of_week, start_time, end_time, is_active)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (therapist_id, day_of_week)
            DO UPDATE SET start_time = $3, end_time = $4, is_active = $5
            RETURNING id, therapist_id, day_of_week, start_time, end_time, is_active"
        )
        .bind(therapist_id)
        .bind(day_of_week)
        .bind(start_time)
        .bind(end_time)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IamError::Database(e.to_string()))
    }
}
