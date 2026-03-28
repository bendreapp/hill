use std::sync::Arc;
use uuid::Uuid;

use crate::clients::domain::entity::*;
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

    pub async fn get_client(&self, id: Uuid) -> Result<Client, ClientError> {
        self.client_repo
            .find_by_id(id)
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
        input: &UpdateClientInput,
    ) -> Result<Client, ClientError> {
        // Ensure client exists
        self.client_repo
            .find_by_id(id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.client_repo.update(id, input).await
    }

    pub async fn soft_delete_client(&self, id: Uuid) -> Result<(), ClientError> {
        self.client_repo
            .find_by_id(id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.client_repo.soft_delete(id).await
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<Client, ClientError> {
        self.client_repo
            .find_by_id(id)
            .await?
            .ok_or(ClientError::ClientNotFound)?;

        self.client_repo.update_status(id, status).await
    }

    pub async fn count_active(&self, therapist_id: Uuid) -> Result<i64, ClientError> {
        self.client_repo.count_active(therapist_id).await
    }
}

// ─── Client Portal Service ──────────────────────────────────────────────────

pub struct ClientPortalService {
    pub portal_repo: Arc<dyn ClientPortalRepository>,
}

impl ClientPortalService {
    pub fn new(portal_repo: Arc<dyn ClientPortalRepository>) -> Self {
        Self { portal_repo }
    }

    pub async fn list_profiles(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ClientPortalProfile>, ClientError> {
        self.portal_repo.list_profiles_by_user(user_id).await
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
}
