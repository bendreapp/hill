use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Asia::Kolkata;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Authenticated user extracted from Supabase JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

/// Practice membership context for multi-therapist access.
#[derive(Debug, Clone)]
pub struct PracticeContext {
    pub practice_id: Uuid,
    pub user_id: Uuid,
    pub role: PracticeRole,
    pub can_view_notes: bool,
    pub accessible_therapist_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeRole {
    Owner,
    Admin,
    Therapist,
}

/// Cursor-based pagination input.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CursorPagination {
    pub cursor: Option<Uuid>,
    pub limit: Option<i64>,
}

impl CursorPagination {
    pub fn limit_or_default(&self) -> i64 {
        self.limit.unwrap_or(20).min(100)
    }
}

/// Offset-based pagination input.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct OffsetPagination {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl OffsetPagination {
    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page();
        (page - 1) * per_page
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(20).min(100)
    }
}

/// Paginated response wrapper.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct Paginated<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Date range query parameter.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// Get current time in IST.
pub fn now_ist() -> DateTime<chrono_tz::Tz> {
    Utc::now().with_timezone(&Kolkata)
}

/// Convert a NaiveDate + NaiveTime in IST to UTC DateTime.
pub fn ist_to_utc(date: NaiveDate, time: NaiveTime) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;
    Kolkata
        .from_local_datetime(&date.and_time(time))
        .earliest()
        .map(|dt| dt.with_timezone(&Utc))
}
