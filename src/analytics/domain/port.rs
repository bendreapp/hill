use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use super::entity::*;
use super::error::AnalyticsError;

#[async_trait]
pub trait AnalyticsQueryPort: Send + Sync {
    async fn overview(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<OverviewStats, AnalyticsError>;

    async fn revenue_by_month(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RevenueByMonth>, AnalyticsError>;

    async fn sessions_by_month(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<SessionsByMonth>, AnalyticsError>;

    async fn client_growth(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ClientGrowth>, AnalyticsError>;

    async fn top_clients(
        &self,
        therapist_ids: &[Uuid],
        start: NaiveDate,
        end: NaiveDate,
        limit: i64,
    ) -> Result<Vec<TopClient>, AnalyticsError>;

    async fn client_category_breakdown(
        &self,
        therapist_ids: &[Uuid],
    ) -> Result<Vec<CategoryBreakdown>, AnalyticsError>;
}
