use std::sync::Arc;
use uuid::Uuid;

use crate::clients::domain::entity::CreateClientInput;
use crate::clients::domain::port::ClientRepository;
use crate::leads::domain::entity::*;
use crate::leads::domain::error::LeadsError;
use crate::leads::domain::port::*;
use crate::shared::email::{send_email, EmailParams};

// ─── Lead Service ──────────────────────────────────────────────────────────

pub struct LeadService {
    pub lead_repo: Arc<dyn LeadRepository>,
    pub client_repo: Arc<dyn ClientRepository>,
    pub resend_api_key: String,
    pub frontend_url: String,
}

impl LeadService {
    pub fn new(
        lead_repo: Arc<dyn LeadRepository>,
        client_repo: Arc<dyn ClientRepository>,
        resend_api_key: String,
        frontend_url: String,
    ) -> Self {
        Self { lead_repo, client_repo, resend_api_key, frontend_url }
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

    /// Public: create a lead from the booking page inquiry form (no auth, resolved by slug).
    /// Also sends auto-ack email to lead and notifies therapist (best-effort).
    pub async fn create_lead_by_slug(
        &self,
        slug: &str,
        input: &CreateLeadInput,
    ) -> Result<Lead, LeadsError> {
        let therapist_id = self
            .lead_repo
            .find_therapist_id_by_slug(slug)
            .await?
            .ok_or(LeadsError::LeadNotFound)?;

        let mut input_with_source = input.clone();
        input_with_source.source = Some("booking_page".to_string());
        let lead = self.lead_repo.create(therapist_id, &input_with_source).await?;

        // Best-effort email notifications — failures are logged but don't fail the request
        let therapist_info = self.lead_repo.find_therapist_info(therapist_id).await;

        match therapist_info {
            Ok(Some(info)) => {
                let therapist_display = info.display().to_string();

                // 1. Auto-ack email to the lead
                if let Some(ref lead_email) = lead.email {
                    let html = format!(
                        "<p>Hi {},</p>\
                         <p>Thanks for reaching out to <strong>{}</strong>. \
                         They'll review your message and get back to you shortly.</p>\
                         <p>— The Bendre Team</p>",
                        lead.full_name, therapist_display
                    );
                    let params = EmailParams {
                        to: lead_email,
                        reply_to: info.email.as_deref(),
                        from_name: &format!("{} via Bendre", therapist_display),
                        subject: "Thanks for reaching out",
                        html: &html,
                    };
                    if let Err(e) = send_email(&self.resend_api_key, params).await {
                        tracing::warn!("Failed to send inquiry ack email: {}", e);
                    }
                }

                // 2. Notify therapist if comms_email is on
                if info.comms_email {
                    if let Some(ref therapist_email) = info.email {
                        let html = format!(
                            "<p>Hi {},</p>\
                             <p>You have a new lead inquiry from <strong>{}</strong>.</p>\
                             <p><a href=\"{}/dashboard/leads\">Log in to review</a></p>\
                             <p>— Bendre</p>",
                            therapist_display, lead.full_name, self.frontend_url
                        );
                        let params = EmailParams {
                            to: therapist_email,
                            reply_to: None,
                            from_name: "Bendre",
                            subject: &format!("New inquiry from {}", lead.full_name),
                            html: &html,
                        };
                        if let Err(e) = send_email(&self.resend_api_key, params).await {
                            tracing::warn!("Failed to send new-lead notification to therapist: {}", e);
                        }
                    }
                }

                // 3. WhatsApp notification — TODO when Gupshup integration is ready
                if info.comms_whatsapp {
                    tracing::info!(
                        "TODO: send WhatsApp notification to therapist {} for new lead {}",
                        therapist_id, lead.id
                    );
                }
            }
            Ok(None) => {
                tracing::warn!("Therapist {} not found when sending lead notifications", therapist_id);
            }
            Err(e) => {
                tracing::warn!("Failed to fetch therapist info for notifications: {}", e);
            }
        }

        Ok(lead)
    }

    /// Convert a lead to a client record.
    /// Creates the client row, updates the lead to status=converted, sends ack email.
    pub async fn convert_to_client(
        &self,
        lead_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<ConvertLeadResponse, LeadsError> {
        // Fetch and verify lead ownership
        let lead = self
            .lead_repo
            .find_by_id(lead_id, therapist_id)
            .await?
            .ok_or(LeadsError::LeadNotFound)?;

        // Create client record from lead data
        let client_input = CreateClientInput {
            full_name: lead.full_name.clone(),
            email: lead.email.clone(),
            phone: lead.phone.clone(),
            date_of_birth: None,
            emergency_contact: None,
            notes_private: None,
            client_type: None,  // defaults to "irregular"
            category: None,     // defaults to "indian"
        };

        let client = self.client_repo.create(therapist_id, &client_input).await
            .map_err(|e| LeadsError::ClientCreationFailed(e.to_string()))?;

        // Mark the lead as converted and link to client
        self.lead_repo.mark_converted(lead_id, therapist_id, client.id).await?;

        // Best-effort: send ack email to lead
        if let Some(ref lead_email) = lead.email {
            let therapist_info = self.lead_repo.find_therapist_info(therapist_id).await;
            let therapist_display = therapist_info
                .ok()
                .flatten()
                .map(|i| i.display().to_string())
                .unwrap_or_else(|| "Your therapist".to_string());

            let html = format!(
                "<p>Hi {},</p>\
                 <p>Thanks for reaching out to <strong>{}</strong>. \
                 We'll send your portal invite soon so you can access your sessions and resources.</p>\
                 <p>— The Bendre Team</p>",
                lead.full_name, therapist_display
            );
            let params = EmailParams {
                to: lead_email,
                reply_to: None,
                from_name: "Bendre",
                subject: "You've been added as a client",
                html: &html,
            };
            if let Err(e) = send_email(&self.resend_api_key, params).await {
                tracing::warn!("Failed to send conversion ack email to lead {}: {}", lead_id, e);
            }
        }

        Ok(ConvertLeadResponse {
            client_id: client.id,
            lead_id,
            status: "converted".to_string(),
        })
    }
}

// ─── Client Invitation Service ─────────────────────────────────────────────

pub struct ClientInvitationService {
    pub invitation_repo: Arc<dyn ClientInvitationRepository>,
    pub resend_api_key: String,
    pub frontend_url: String,
}

impl ClientInvitationService {
    pub fn new(
        invitation_repo: Arc<dyn ClientInvitationRepository>,
        resend_api_key: String,
        frontend_url: String,
    ) -> Self {
        Self { invitation_repo, resend_api_key, frontend_url }
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

    /// Send a portal invite email to a client.
    /// Creates invitation if one doesn't exist, sends email, marks invite_sent_at.
    pub async fn send_portal_invite(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        client_email: &str,
        client_name: &str,
        therapist_display_name: &str,
    ) -> Result<SendPortalInviteResponse, LeadsError> {
        // Get or create invitation
        let invitation = self
            .create_invitation(therapist_id, client_id, Some(client_email), None)
            .await?;

        // Send the portal invite email
        let portal_url = format!("{}/onboard/{}", self.frontend_url, invitation.token);
        let html = format!(
            "<p>Hi {},</p>\
             <p><strong>{}</strong> has invited you to access your client portal on Bendre.</p>\
             <p>Click below to set up your access:</p>\
             <p><a href=\"{}\" style=\"background:#5C7A6B;color:white;padding:10px 20px;border-radius:6px;text-decoration:none;display:inline-block\">Set up portal access</a></p>\
             <p>This link expires in 7 days.</p>\
             <p>— The Bendre Team</p>",
            client_name, therapist_display_name, portal_url
        );
        let params = EmailParams {
            to: client_email,
            reply_to: None,
            from_name: &format!("{} via Bendre", therapist_display_name),
            subject: "You're invited to your client portal",
            html: &html,
        };

        if let Err(e) = send_email(&self.resend_api_key, params).await {
            tracing::warn!("Failed to send portal invite email: {}", e);
            // Return error since this is a primary action (therapist explicitly clicked Send Invite)
            return Err(LeadsError::EmailFailed(e));
        }

        // Mark invite as sent
        let updated = self.invitation_repo.mark_invite_sent(invitation.id).await?;
        let sent_at = updated.invite_sent_at.unwrap_or_else(chrono::Utc::now);

        Ok(SendPortalInviteResponse {
            invitation_id: updated.id,
            token: updated.token,
            sent_at,
        })
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

    /// Returns enriched invitation details (client name/email, therapist name/avatar).
    /// Does NOT validate status — allows showing a "claimed" or "expired" state on the page.
    pub async fn get_detail_by_token(&self, token: &str) -> Result<serde_json::Value, LeadsError> {
        let detail = self
            .invitation_repo
            .find_detail_by_token(token)
            .await?
            .ok_or(LeadsError::InvitationNotFound)?;

        let is_usable = detail.status == "pending" && detail.claimed_at.is_none();
        Ok(serde_json::json!({
            "id": detail.id,
            "token": detail.token,
            "status": detail.status,
            "is_usable": is_usable,
            "expires_at": detail.expires_at,
            "client_full_name": detail.client_full_name,
            "client_email": detail.client_email,
            "therapist_name": detail.therapist_name,
            "therapist_avatar_url": detail.therapist_avatar_url,
            "therapist_slug": detail.therapist_slug,
        }))
    }

    pub async fn claim(&self, token: &str) -> Result<ClientInvitation, LeadsError> {
        let invitation = self.get_by_token(token).await?;
        if invitation.claimed_at.is_some() {
            return Err(LeadsError::InvitationAlreadyClaimed);
        }
        self.invitation_repo.claim(token).await
    }
}
