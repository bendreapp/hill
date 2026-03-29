use std::sync::Arc;
use uuid::Uuid;

use crate::leads::domain::entity::*;
use crate::leads::domain::error::LeadsError;
use crate::leads::domain::port::*;

// ─── Lead Service ──────────────────────────────────────────────────────────

pub struct LeadService {
    pub lead_repo: Arc<dyn LeadRepository>,
}

impl LeadService {
    pub fn new(lead_repo: Arc<dyn LeadRepository>) -> Self {
        Self { lead_repo }
    }

    pub async fn create_lead(
        &self,
        therapist_id: Uuid,
        input: &CreateLeadInput,
    ) -> Result<Lead, LeadsError> {
        self.lead_repo.create(therapist_id, input).await
    }

    pub async fn get_lead(&self, id: Uuid, therapist_id: Uuid) -> Result<Lead, LeadsError> {
        self.lead_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(LeadsError::LeadNotFound)
    }

    pub async fn list_leads(
        &self,
        therapist_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Lead>, i64), LeadsError> {
        self.lead_repo
            .list_by_therapist(therapist_id, status, limit, offset)
            .await
    }

    pub async fn update_lead(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateLeadInput,
    ) -> Result<Lead, LeadsError> {
        self.lead_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(LeadsError::LeadNotFound)?;
        self.lead_repo.update(id, therapist_id, input).await
    }
}

// ─── Client Invitation Service ─────────────────────────────────────────────

pub struct ClientInvitationService {
    pub invitation_repo: Arc<dyn ClientInvitationRepository>,
}

impl ClientInvitationService {
    pub fn new(invitation_repo: Arc<dyn ClientInvitationRepository>) -> Self {
        Self { invitation_repo }
    }

    pub async fn create_invitation(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> Result<ClientInvitation, LeadsError> {
        // Check if there's already a pending invitation
        if let Some(existing) = self.invitation_repo.find_by_client(client_id).await? {
            if existing.status == "pending" && existing.expires_at > chrono::Utc::now() {
                return Ok(existing); // Return existing active invitation
            }
        }
        self.invitation_repo
            .create(therapist_id, client_id, email, phone)
            .await
    }

    pub async fn get_by_token(&self, token: &str) -> Result<ClientInvitation, LeadsError> {
        let invitation = self
            .invitation_repo
            .find_by_token(token)
            .await?
            .ok_or(LeadsError::InvitationNotFound)?;

        if invitation.status != "pending" {
            return Err(LeadsError::InvitationAlreadyClaimed);
        }
        if invitation.expires_at < chrono::Utc::now() {
            return Err(LeadsError::InvitationExpired);
        }
        Ok(invitation)
    }

    pub async fn claim(&self, token: &str) -> Result<ClientInvitation, LeadsError> {
        let invitation = self.get_by_token(token).await?;
        if invitation.claimed_at.is_some() {
            return Err(LeadsError::InvitationAlreadyClaimed);
        }
        self.invitation_repo.claim(token).await
    }
}
