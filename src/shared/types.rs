use serde::Serialize;
use uuid::Uuid;

/// Authenticated user extracted from Supabase JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    #[allow(dead_code)]
    pub role: String,
}


/// Paginated response wrapper.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct Paginated<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

