use async_trait::async_trait;
use uuid::Uuid;

use super::entity::{
    Client, ClientPortalProfile, ClientSessionType, CreateClientSessionTypeInput,
    PortalSession, UpdateClientSessionTypeInput,
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
}
