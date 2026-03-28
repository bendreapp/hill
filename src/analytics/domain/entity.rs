
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OverviewStats {
    pub total_clients: i64,
    pub active_clients: i64,
    pub total_sessions: i64,
    pub completed_sessions: i64,
    pub cancelled_sessions: i64,
    pub no_show_sessions: i64,
    pub total_revenue_inr: i64,
    pub outstanding_inr: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct RevenueByMonth {
    pub month: String,
    pub paid_inr: i64,
    pub outstanding_inr: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct SessionsByMonth {
    pub month: String,
    pub completed: i64,
    pub cancelled: i64,
    pub no_show: i64,
    pub scheduled: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ClientGrowth {
    pub month: String,
    pub new_clients: i64,
    pub cumulative: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct TopClient {
    pub client_id: Uuid,
    pub full_name: String,
    pub session_count: i64,
    pub total_paid_inr: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct CategoryBreakdown {
    pub category: String,
    pub count: i64,
}
