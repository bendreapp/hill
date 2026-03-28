use async_trait::async_trait;
use uuid::Uuid;

use super::entity::{Client, ClientPortalProfile, PortalSession};
use super::error::ClientError;

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>, ClientError>;

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
        input: &super::entity::UpdateClientInput,
    ) -> Result<Client, ClientError>;

    async fn soft_delete(&self, id: Uuid) -> Result<(), ClientError>;

    async fn update_status(&self, id: Uuid, status: &str) -> Result<Client, ClientError>;

    async fn find_by_email(
        &self,
        therapist_id: Uuid,
        email: &str,
    ) -> Result<Option<Client>, ClientError>;

    async fn count_active(&self, therapist_id: Uuid) -> Result<i64, ClientError>;
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
