use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::analytics::domain::entity::*;
use crate::analytics::domain::error::AnalyticsError;
use crate::analytics::domain::port::AnalyticsQueryPort;

pub struct PgAnalyticsRepository {
    pool: PgPool,
}

impl PgAnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionCounts {
    total: Option<i64>,
    completed: Option<i64>,
    cancelled: Option<i64>,
    no_show: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct RevenueSums {
    paid: Option<i64>,
    outstanding: Option<i64>,
}

#[async_trait]
impl AnalyticsQueryPort for PgAnalyticsRepository {
    async fn overview(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<OverviewStats, AnalyticsError> {
        let total_clients = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM clients
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL"
        )
        .bind(therapist_ids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        let active_clients = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM clients
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL AND status::text = 'active'"
        )
        .bind(therapist_ids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        let sc = sqlx::query_as::<_, SessionCounts>(
            "SELECT
                count(*) as total,
                count(*) FILTER (WHERE status::text = 'completed') as completed,
                count(*) FILTER (WHERE status::text = 'cancelled') as cancelled,
                count(*) FILTER (WHERE status::text = 'no_show') as no_show
            FROM sessions
            WHERE therapist_id = ANY($1)
              AND deleted_at IS NULL
              AND starts_at >= $2::date
              AND starts_at < ($3::date + interval '1 day')"
        )
        .bind(therapist_ids)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        let rv = sqlx::query_as::<_, RevenueSums>(
            "SELECT
                COALESCE(sum(total_inr) FILTER (WHERE status::text = 'paid'), 0) as paid,
                COALESCE(sum(total_inr) FILTER (WHERE status::text = 'unpaid'), 0) as outstanding
            FROM invoices
            WHERE therapist_id = ANY($1)
              AND created_at >= $2::date
              AND created_at < ($3::date + interval '1 day')"
        )
        .bind(therapist_ids)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(OverviewStats {
            total_clients,
            active_clients,
            total_sessions: sc.total.unwrap_or(0),
            completed_sessions: sc.completed.unwrap_or(0),
            cancelled_sessions: sc.cancelled.unwrap_or(0),
            no_show_sessions: sc.no_show.unwrap_or(0),
            total_revenue_inr: rv.paid.unwrap_or(0),
            outstanding_inr: rv.outstanding.unwrap_or(0),
        })
    }

    async fn revenue_by_month(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RevenueByMonth>, AnalyticsError> {
        let rows = sqlx::query_as::<_, RevenueByMonth>(
            "SELECT
                to_char(created_at, 'YYYY-MM') as month,
                COALESCE(sum(total_inr) FILTER (WHERE status::text = 'paid'), 0) as paid_inr,
                COALESCE(sum(total_inr) FILTER (WHERE status::text = 'unpaid'), 0) as outstanding_inr
            FROM invoices
            WHERE therapist_id = ANY($1)
              AND created_at >= $2::date
              AND created_at < ($3::date + interval '1 day')
            GROUP BY to_char(created_at, 'YYYY-MM')
            ORDER BY month ASC"
        )
        .bind(therapist_ids)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn sessions_by_month(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<SessionsByMonth>, AnalyticsError> {
        let rows = sqlx::query_as::<_, SessionsByMonth>(
            "SELECT
                to_char(starts_at, 'YYYY-MM') as month,
                count(*) FILTER (WHERE status::text = 'completed') as completed,
                count(*) FILTER (WHERE status::text = 'cancelled') as cancelled,
                count(*) FILTER (WHERE status::text = 'no_show') as no_show,
                count(*) FILTER (WHERE status::text = 'scheduled') as scheduled
            FROM sessions
            WHERE therapist_id = ANY($1)
              AND deleted_at IS NULL
              AND starts_at >= $2::date
              AND starts_at < ($3::date + interval '1 day')
            GROUP BY to_char(starts_at, 'YYYY-MM')
            ORDER BY month ASC"
        )
        .bind(therapist_ids)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn client_growth(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ClientGrowth>, AnalyticsError> {
        let rows = sqlx::query_as::<_, ClientGrowth>(
            "WITH monthly AS (
                SELECT
                    to_char(created_at, 'YYYY-MM') as month,
                    count(*) as new_clients
                FROM clients
                WHERE therapist_id = ANY($1)
                  AND deleted_at IS NULL
                  AND created_at >= $2::date
                  AND created_at < ($3::date + interval '1 day')
                GROUP BY to_char(created_at, 'YYYY-MM')
            )
            SELECT
                month,
                new_clients,
                COALESCE(sum(new_clients) OVER (ORDER BY month), 0)::bigint as cumulative
            FROM monthly
            ORDER BY month ASC"
        )
        .bind(therapist_ids)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn top_clients(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
        limit: i64,
    ) -> Result<Vec<TopClient>, AnalyticsError> {
        let rows = sqlx::query_as::<_, TopClient>(
            "SELECT
                c.id as client_id,
                c.full_name,
                count(s.id) as session_count,
                COALESCE(sum(i.total_inr) FILTER (WHERE i.status::text = 'paid'), 0) as total_paid_inr
            FROM clients c
            LEFT JOIN sessions s ON s.client_id = c.id
                AND s.deleted_at IS NULL
                AND s.starts_at >= $2::date
                AND s.starts_at < ($3::date + interval '1 day')
            LEFT JOIN invoices i ON i.client_id = c.id
                AND i.created_at >= $2::date
                AND i.created_at < ($3::date + interval '1 day')
            WHERE c.therapist_id = ANY($1)
              AND c.deleted_at IS NULL
            GROUP BY c.id, c.full_name
            ORDER BY session_count DESC
            LIMIT $4"
        )
        .bind(therapist_ids)
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn client_category_breakdown(
        &self,
        therapist_ids: &[Uuid],
    ) -> Result<Vec<CategoryBreakdown>, AnalyticsError> {
        let rows = sqlx::query_as::<_, CategoryBreakdown>(
            "SELECT
                category::text as category,
                count(*) as count
            FROM clients
            WHERE therapist_id = ANY($1) AND deleted_at IS NULL
            GROUP BY category
            ORDER BY count DESC"
        )
        .bind(therapist_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::Database(e.to_string()))?;

        Ok(rows)
    }
}
