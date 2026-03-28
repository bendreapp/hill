use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Client {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub emergency_contact: Option<String>,
    pub notes_private: Option<String>,
    pub intake_completed: bool,
    pub is_active: bool,
    pub status: String,
    pub client_type: String,
    pub category: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateClientInput {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub emergency_contact: Option<String>,
    pub notes_private: Option<String>,
    pub client_type: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateClientInput {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub emergency_contact: Option<String>,
    pub notes_private: Option<String>,
    pub intake_completed: Option<bool>,
    pub client_type: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateStatusInput {
    pub status: String,
}

/// A lightweight view of a client profile for the portal.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ClientPortalProfile {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub intake_completed: bool,
    pub status: String,
}

/// A compact session view returned in the client portal.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct PortalSession {
    pub id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub duration_mins: i32,
    pub status: String,
}
