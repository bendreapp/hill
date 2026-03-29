use async_trait::async_trait;
use uuid::Uuid;

use super::entity::*;
use super::error::LeadsError;

#[async_trait]
pub trait LeadRepository: Send + Sync {
    async fn create(&self, therapist_id: Uuid, input: &CreateLeadInput) -> Result<Lead, LeadsError>;
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<Lead>, LeadsError>;
    async fn list_by_therapist(&self, therapist_id: Uuid, status: Option<&str>, limit: i64, offset: i64) -> Result<(Vec<Lead>, i64), LeadsError>;
    async fn update(&self, id: Uuid, therapist_id: Uuid, input: &UpdateLeadInput) -> Result<Lead, LeadsError>;
}

#[async_trait]
pub trait ClientInvitationRepository: Send + Sync {
    async fn create(&self, therapist_id: Uuid, client_id: Uuid, email: Option<&str>, phone: Option<&str>) -> Result<ClientInvitation, LeadsError>;
    async fn find_by_token(&self, token: &str) -> Result<Option<ClientInvitation>, LeadsError>;
    async fn find_by_client(&self, client_id: Uuid) -> Result<Option<ClientInvitation>, LeadsError>;
    async fn claim(&self, token: &str) -> Result<ClientInvitation, LeadsError>;
}
