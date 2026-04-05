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
    async fn find_therapist_id_by_slug(&self, slug: &str) -> Result<Option<Uuid>, LeadsError>;
    /// Mark lead as converted and link to the created client record
    async fn mark_converted(&self, lead_id: Uuid, therapist_id: Uuid, client_id: Uuid) -> Result<Lead, LeadsError>;
    /// Fetch the therapist's email from auth.users (for notification emails)
    async fn find_therapist_email(&self, therapist_id: Uuid) -> Result<Option<String>, LeadsError>;
    /// Fetch therapist info (name + comms flags) needed for lead processing
    async fn find_therapist_info(&self, therapist_id: Uuid) -> Result<Option<TherapistInfo>, LeadsError>;
}

/// Minimal therapist info needed for email notifications
#[derive(Debug, Clone)]
pub struct TherapistInfo {
    pub full_name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub comms_email: bool,
    pub comms_whatsapp: bool,
}

impl TherapistInfo {
    pub fn display(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.full_name)
    }
}

#[async_trait]
pub trait ClientInvitationRepository: Send + Sync {
    async fn create(&self, therapist_id: Uuid, client_id: Uuid, email: Option<&str>, phone: Option<&str>) -> Result<ClientInvitation, LeadsError>;
    async fn find_by_token(&self, token: &str) -> Result<Option<ClientInvitation>, LeadsError>;
    async fn find_by_client(&self, client_id: Uuid) -> Result<Option<ClientInvitation>, LeadsError>;
    async fn claim(&self, token: &str) -> Result<ClientInvitation, LeadsError>;
    /// Mark the invitation as having had its email sent (sets invite_sent_at = now())
    async fn mark_invite_sent(&self, invitation_id: Uuid) -> Result<ClientInvitation, LeadsError>;
}
