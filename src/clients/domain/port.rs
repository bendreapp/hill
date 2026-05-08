use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::entity::{
    Client, ClientPortalProfile, ClientSessionType, CreateClientSessionTypeInput,
    PortalInvoice, PortalSession, UpdateClientSessionTypeInput, UpdatePortalProfileInput,
};
use super::error::ClientError;

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<Client>, ClientError>;

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Client>, i64), ClientError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &super::entity::CreateClientInput,
    ) -> Result<Client, ClientError>;

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &super::entity::UpdateClientInput,
    ) -> Result<Client, ClientError>;

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClientError>;

    async fn update_status(&self, id: Uuid, therapist_id: Uuid, status: &str) -> Result<Client, ClientError>;

    async fn find_by_email(
        &self,
        therapist_id: Uuid,
        email: &str,
    ) -> Result<Option<Client>, ClientError>;

    async fn count_active(&self, therapist_id: Uuid) -> Result<i64, ClientError>;

    /// Link a Supabase auth user_id to a client record (called during portal claim).
    async fn link_user_id(&self, client_id: Uuid, user_id: Uuid) -> Result<(), ClientError>;
}

#[async_trait]
pub trait ClientSessionTypeRepository: Send + Sync {
    async fn list_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Vec<ClientSessionType>, ClientError>;

    async fn find_by_id(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Option<ClientSessionType>, ClientError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        input: &CreateClientSessionTypeInput,
    ) -> Result<ClientSessionType, ClientError>;

    async fn update(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
        input: &UpdateClientSessionTypeInput,
    ) -> Result<ClientSessionType, ClientError>;

    async fn delete(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), ClientError>;

    /// Sets is_default=false on all session types for this client, then is_default=true on target.
    async fn set_default(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), ClientError>;
}

#[async_trait]
pub trait ClientPortalRepository: Send + Sync {
    async fn list_profiles_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ClientPortalProfile>, ClientError>;

    async fn update_profile(
        &self,
        client_id: Uuid,
        user_id: Uuid,
        input: &UpdatePortalProfileInput,
    ) -> Result<ClientPortalProfile, ClientError>;

    async fn upcoming_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PortalSession>, ClientError>;

    async fn past_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PortalSession>, i64), ClientError>;

    async fn list_invoices(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<PortalInvoice>, ClientError>;

    async fn request_reschedule(
        &self,
        session_id: Uuid,
        client_id: Uuid,
        requested_date: &str,
        requested_time: &str,
        reason: Option<&str>,
    ) -> Result<(), ClientError>;

    async fn find_session_by_id_for_client(
        &self,
        session_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<PortalSession>, ClientError>;

    async fn list_intake_forms(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, ClientError>;

    async fn list_sessions_for_reminder(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<ReminderSession>, ClientError>;

    async fn mark_reminder_sent(
        &self,
        session_id: Uuid,
    ) -> Result<(), ClientError>;

    async fn clear_reschedule_request(
        &self,
        session_id: Uuid,
    ) -> Result<(), ClientError>;

    async fn get_client_email_by_id(
        &self,
        client_id: Uuid,
    ) -> Result<Option<String>, ClientError>;
}

/// Minimal session data needed to send reminders.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReminderSession {
    pub id: Uuid,
    #[allow(dead_code)]
    pub therapist_id: Uuid,
    #[allow(dead_code)]
    pub client_id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub client_email: Option<String>,
    pub client_name: String,
    pub therapist_email: Option<String>,
    pub therapist_name: String,
}
