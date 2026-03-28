use std::sync::Arc;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::analytics::domain::entity::*;
use crate::analytics::domain::error::AnalyticsError;
use crate::analytics::domain::port::AnalyticsQueryPort;

pub struct AnalyticsService {
    pub query_port: Arc<dyn AnalyticsQueryPort>,
}

impl AnalyticsService {
    pub fn new(query_port: Arc<dyn AnalyticsQueryPort>) -> Self {
        Self { query_port }
    }

    pub async fn overview(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<OverviewStats, AnalyticsError> {
        self.query_port.overview(therapist_ids, start, end).await
    }

    pub async fn revenue_by_month(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RevenueByMonth>, AnalyticsError> {
        self.query_port
            .revenue_by_month(therapist_ids, start, end)
            .await
    }

    pub async fn sessions_by_month(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<SessionsByMonth>, AnalyticsError> {
        self.query_port
            .sessions_by_month(therapist_ids, start, end)
            .await
    }

    pub async fn client_growth(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ClientGrowth>, AnalyticsError> {
        self.query_port
            .client_growth(therapist_ids, start, end)
            .await
    }

    pub async fn top_clients(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
        limit: i64,
    ) -> Result<Vec<TopClient>, AnalyticsError> {
        self.query_port
            .top_clients(therapist_ids, start, end, limit)
            .await
    }

    pub async fn client_category_breakdown(
        &self,
        therapist_ids: &[Uuid],
    ) -> Result<Vec<CategoryBreakdown>, AnalyticsError> {
        self.query_port
            .client_category_breakdown(therapist_ids)
            .await
    }
}
