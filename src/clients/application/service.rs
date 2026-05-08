use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
#[allow(unused_imports)]
use chrono_tz;

use crate::clients::domain::entity::{
    Client, ClientPortalProfile, ClientSessionType, CreateClientInput,
    CreateClientSessionTypeInput, PortalInvoice, PortalRescheduleInput,
    PortalSession, UpdateClientInput, UpdateClientSessionTypeInput, UpdatePortalProfileInput,
};
use crate::clients::domain::error::ClientError;
use crate::clients::domain::port::*;

// ─── Client Service ─────────────────────────────────────────────────────────

pub struct ClientService {
    pub client_repo: Arc<dyn ClientRepository>,
}

impl ClientService {
    pub fn new(client_repo: Arc<dyn ClientRepository>) -> Self {
        Self { client_repo }
    }

    pub async fn get_client(&self, id: Uuid, therapist_id: Uuid) -> Result<Client, ClientError> {
        self.client_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClientError::ClientNotFound)
    }

    pub async fn list_clients(
        &self,
        therapist_ids: &[Uuid],
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Client>, i64), ClientError> {
        self.client_repo
            .list(therapist_ids, status_filter, limit, offset)
            .await
    }

    pub async fn create_client(
        &self,
        therapist_id: Uuid,
        input: &CreateClientInput,
        plan_max_clients: Option<i64>,
    ) -> Result<Client, ClientError> {
        // Enforce plan tier limit
        if let Some(max) = plan_max_clients {
            let current = self.client_repo.count_active(therapist_id).await?;
            if current >= max {
                return Err(ClientError::PlanLimitExceeded(max));
            }
        }

        // Check for duplicate email
        if let Some(ref email) = input.email {
            if !email.is_empty() {
                if self
                    .client_repo
                    .find_by_email(therapist_id, email)
                    .await?
                    .is_some()
                {
                    return Err(ClientError::DuplicateEmail);
                }
            }
        }

        self.client_repo.create(therapist_id, input).await
    }

    pub async fn update_client(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateClientInput,
    ) -> Result<Client, ClientError> {
        // Ensure client exists and belongs to this therapist
        self.client_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.client_repo.update(id, therapist_id, input).await
    }

    pub async fn soft_delete_client(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClientError> {
        self.client_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.client_repo.soft_delete(id, therapist_id).await
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        status: &str,
    ) -> Result<Client, ClientError> {
        self.client_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.client_repo.update_status(id, therapist_id, status).await
    }

    pub async fn count_active(&self, therapist_id: Uuid) -> Result<i64, ClientError> {
        self.client_repo.count_active(therapist_id).await
    }

    /// Link a Supabase auth user_id to a client record.
    /// Called after a client claims their portal invitation and creates a Supabase account.
    pub async fn link_user_id(&self, client_id: Uuid, user_id: Uuid) -> Result<(), ClientError> {
        self.client_repo.link_user_id(client_id, user_id).await
    }
}

// ─── Client Session Type Service ────────────────────────────────────────────

pub struct ClientSessionTypeService {
    pub repo: Arc<dyn crate::clients::domain::port::ClientSessionTypeRepository>,
}

impl ClientSessionTypeService {
    pub fn new(repo: Arc<dyn crate::clients::domain::port::ClientSessionTypeRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_for_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Vec<ClientSessionType>, ClientError> {
        self.repo.list_by_client(client_id, therapist_id).await
    }

    pub async fn create_for_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        input: &CreateClientSessionTypeInput,
    ) -> Result<ClientSessionType, ClientError> {
        self.repo.create(therapist_id, client_id, input).await
    }

    pub async fn update_for_client(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
        input: &UpdateClientSessionTypeInput,
    ) -> Result<ClientSessionType, ClientError> {
        self.repo
            .find_by_id(id, client_id, therapist_id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.repo.update(id, client_id, therapist_id, input).await
    }

    pub async fn delete_for_client(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), ClientError> {
        self.repo.delete(id, client_id, therapist_id).await
    }

    pub async fn set_default(
        &self,
        id: Uuid,
        client_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), ClientError> {
        self.repo.set_default(id, client_id, therapist_id).await
    }
}

// ─── Client Portal Service ──────────────────────────────────────────────────

pub struct ClientPortalService {
    pub portal_repo: Arc<dyn ClientPortalRepository>,
    pub resend_api_key: String,
}

impl ClientPortalService {
    pub fn new(portal_repo: Arc<dyn ClientPortalRepository>, resend_api_key: String) -> Self {
        Self { portal_repo, resend_api_key }
    }

    pub async fn list_profiles(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ClientPortalProfile>, ClientError> {
        self.portal_repo.list_profiles_by_user(user_id).await
    }

    /// Verify that the given client_id belongs to the given user_id.
    /// Returns the matching profile so callers can reuse it.
    pub async fn verify_client_ownership(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<(), ClientError> {
        let profiles = self.portal_repo.list_profiles_by_user(user_id).await?;
        if !profiles.iter().any(|p| p.id == client_id) {
            return Err(ClientError::ClientNotFound);
        }
        Ok(())
    }

    pub async fn update_profile(
        &self,
        client_id: Uuid,
        user_id: Uuid,
        input: &UpdatePortalProfileInput,
    ) -> Result<ClientPortalProfile, ClientError> {
        self.portal_repo.update_profile(client_id, user_id, input).await
    }

    pub async fn list_invoices(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<PortalInvoice>, ClientError> {
        self.verify_client_ownership(user_id, client_id).await?;
        self.portal_repo.list_invoices(client_id).await
    }

    pub async fn request_reschedule(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        session_id: Uuid,
        input: &PortalRescheduleInput,
    ) -> Result<(), ClientError> {
        self.verify_client_ownership(user_id, client_id).await?;
        self.portal_repo
            .request_reschedule(
                session_id,
                client_id,
                &input.requested_date,
                &input.requested_time,
                input.reason.as_deref(),
            )
            .await
    }

    pub async fn list_intake_forms(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, ClientError> {
        self.verify_client_ownership(user_id, client_id).await?;
        self.portal_repo.list_intake_forms(client_id).await
    }

    pub async fn upcoming_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PortalSession>, ClientError> {
        self.portal_repo.upcoming_sessions(client_id, limit).await
    }

    pub async fn past_sessions(
        &self,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PortalSession>, i64), ClientError> {
        self.portal_repo
            .past_sessions(client_id, limit, offset)
            .await
    }

    /// Send 24h session reminders to clients and therapists.
    /// Queries sessions starting between `from` and `to` UTC that haven't been reminded yet.
    /// Best-effort: email failures are logged but do not abort the loop.
    pub async fn send_session_reminders(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<usize, ClientError> {
        let sessions = self.portal_repo.list_sessions_for_reminder(from, to).await?;
        let mut sent = 0usize;

        for session in &sessions {
            let ist_time = session.starts_at.with_timezone(&chrono_tz::Asia::Kolkata);
            let formatted_time = ist_time.format("%d %b %Y at %I:%M %p IST").to_string();

            // Email to client
            if let Some(ref client_email) = session.client_email {
                let body = format!(
                    "Hi {},\n\nThis is a reminder that you have a session with {} tomorrow on {}.\n\nPlease log in to the portal if you need to reschedule.",
                    session.client_name,
                    session.therapist_name,
                    formatted_time,
                );
                let result = crate::shared::email::send_email(
                    &self.resend_api_key,
                    crate::shared::email::EmailParams {
                        to: client_email,
                        reply_to: None,
                        from_name: "Bendre",
                        subject: &format!("Reminder: Session tomorrow at {}", formatted_time),
                        html: &body,
                    },
                )
                .await;
                if let Err(e) = result {
                    tracing::warn!(
                        session_id = %session.id,
                        "Failed to send reminder to client {}: {}",
                        client_email, e
                    );
                }
            }

            // Email to therapist
            if let Some(ref therapist_email) = session.therapist_email {
                let body = format!(
                    "Hi {},\n\nThis is a reminder that you have a session with {} tomorrow on {}.",
                    session.therapist_name,
                    session.client_name,
                    formatted_time,
                );
                let result = crate::shared::email::send_email(
                    &self.resend_api_key,
                    crate::shared::email::EmailParams {
                        to: therapist_email,
                        reply_to: None,
                        from_name: "Bendre",
                        subject: &format!("Reminder: Session with {} tomorrow", session.client_name),
                        html: &body,
                    },
                )
                .await;
                if let Err(e) = result {
                    tracing::warn!(
                        session_id = %session.id,
                        "Failed to send reminder to therapist {}: {}",
                        therapist_email, e
                    );
                }
            }

            // Mark as reminded (best-effort)
            if let Err(e) = self.portal_repo.mark_reminder_sent(session.id).await {
                tracing::error!(session_id = %session.id, "Failed to mark reminder sent: {:?}", e);
            } else {
                sent += 1;
            }
        }

        Ok(sent)
    }
}
