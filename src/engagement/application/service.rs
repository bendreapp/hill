use std::sync::Arc;
use uuid::Uuid;

use crate::engagement::domain::entity::*;
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::*;

// ─── Resource Service ───────────────────────────────────────────────────────

pub struct ResourceService {
    pub resource_repo: Arc<dyn ResourceRepository>,
}

impl ResourceService {
    pub fn new(resource_repo: Arc<dyn ResourceRepository>) -> Self {
        Self { resource_repo }
    }

    pub async fn get_resource(&self, id: Uuid, therapist_id: Uuid) -> Result<Resource, EngagementError> {
        self.resource_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::ResourceNotFound)
    }

    pub async fn list_resources(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Resource>, i64), EngagementError> {
        self.resource_repo.list(therapist_ids, limit, offset).await
    }

    pub async fn create_resource(
        &self,
        therapist_id: Uuid,
        input: &CreateResourceInput,
    ) -> Result<Resource, EngagementError> {
        self.resource_repo.create(therapist_id, input).await
    }

    pub async fn update_resource(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateResourceInput,
    ) -> Result<Resource, EngagementError> {
        self.resource_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::ResourceNotFound)?;
        self.resource_repo.update(id, therapist_id, input).await
    }

    pub async fn delete_resource(&self, id: Uuid, therapist_id: Uuid) -> Result<(), EngagementError> {
        self.resource_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::ResourceNotFound)?;
        self.resource_repo.soft_delete(id, therapist_id).await
    }

    pub async fn share_resource(
        &self,
        resource_id: Uuid,
        therapist_id: Uuid,
        client_ids: &[Uuid],
        note: Option<&str>,
    ) -> Result<Vec<ClientResource>, EngagementError> {
        self.resource_repo
            .find_by_id(resource_id, therapist_id)
            .await?
            .ok_or(EngagementError::ResourceNotFound)?;
        self.resource_repo
            .share(resource_id, therapist_id, client_ids, note)
            .await
    }

    pub async fn unshare_resource(
        &self,
        resource_id: Uuid,
        therapist_id: Uuid,
        client_ids: &[Uuid],
    ) -> Result<(), EngagementError> {
        self.resource_repo.unshare(resource_id, therapist_id, client_ids).await
    }

    pub async fn list_shared_with_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ClientResource>, i64), EngagementError> {
        self.resource_repo
            .list_shared_with_client(client_id, therapist_id, limit, offset)
            .await
    }
}

// ─── Intake Service ─────────────────────────────────────────────────────────

pub struct IntakeService {
    pub form_repo: Arc<dyn IntakeFormRepository>,
    pub encryption: Arc<dyn EngagementEncryptionPort>,
}

impl IntakeService {
    pub fn new(
        form_repo: Arc<dyn IntakeFormRepository>,
        encryption: Arc<dyn EngagementEncryptionPort>,
    ) -> Self {
        Self {
            form_repo,
            encryption,
        }
    }

    pub async fn get_form(&self, id: Uuid, therapist_id: Uuid) -> Result<IntakeForm, EngagementError> {
        self.form_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::IntakeFormNotFound)
    }

    pub async fn list_forms(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IntakeForm>, i64), EngagementError> {
        self.form_repo.list(therapist_ids, limit, offset).await
    }

    pub async fn create_form(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeFormInput,
    ) -> Result<IntakeForm, EngagementError> {
        self.form_repo.create(therapist_id, input).await
    }

    pub async fn update_form(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateIntakeFormInput,
    ) -> Result<IntakeForm, EngagementError> {
        self.form_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::IntakeFormNotFound)?;
        self.form_repo.update(id, therapist_id, input).await
    }

    pub async fn delete_form(&self, id: Uuid, therapist_id: Uuid) -> Result<(), EngagementError> {
        self.form_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::IntakeFormNotFound)?;
        self.form_repo.delete(id, therapist_id).await
    }

    pub async fn create_response(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeResponseInput,
    ) -> Result<IntakeResponse, EngagementError> {
        // Snapshot the form fields at creation time
        let form = self
            .form_repo
            .find_by_id(input.intake_form_id, therapist_id)
            .await?
            .ok_or(EngagementError::IntakeFormNotFound)?;

        self.form_repo
            .create_response(therapist_id, input, &form.fields)
            .await
    }

    pub async fn get_response(&self, id: Uuid, therapist_id: Uuid) -> Result<IntakeResponse, EngagementError> {
        let mut resp = self
            .form_repo
            .find_response_by_id(id, therapist_id)
            .await?
            .ok_or(EngagementError::IntakeResponseNotFound)?;
        self.decrypt_response(&mut resp);
        Ok(resp)
    }

    pub async fn get_response_by_token(
        &self,
        token: Uuid,
    ) -> Result<IntakeResponse, EngagementError> {
        let mut resp = self
            .form_repo
            .find_response_by_token(token)
            .await?
            .ok_or(EngagementError::IntakeResponseNotFound)?;
        self.decrypt_response(&mut resp);
        Ok(resp)
    }

    pub async fn list_responses_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IntakeResponse>, i64), EngagementError> {
        let (mut responses, total) = self
            .form_repo
            .list_responses_by_client(client_id, therapist_id, limit, offset)
            .await?;
        for resp in &mut responses {
            self.decrypt_response(resp);
        }
        Ok((responses, total))
    }

    pub async fn submit_response(
        &self,
        id: Uuid,
        input: &SubmitIntakeResponseInput,
    ) -> Result<IntakeResponse, EngagementError> {
        // Note: submit_response is called from the public (unauthenticated) endpoint.
        // The repo's submit_response query will fail if the id doesn't exist,
        // so we don't need a separate existence check with therapist_id here.
        let encrypted = self.encryption.encrypt(&input.responses)?;
        let mut resp = self.form_repo.submit_response(id, &encrypted).await?;
        self.decrypt_response(&mut resp);
        Ok(resp)
    }

    fn decrypt_response(&self, resp: &mut IntakeResponse) {
        if let Some(ref val) = resp.responses {
            resp.responses = self.encryption.decrypt(val).ok();
        }
    }
}

// ─── Broadcast Service ──────────────────────────────────────────────────────

pub struct BroadcastService {
    pub broadcast: Arc<dyn BroadcastPort>,
}

impl BroadcastService {
    pub fn new(broadcast: Arc<dyn BroadcastPort>) -> Self {
        Self { broadcast }
    }

    pub async fn send_whatsapp(
        &self,
        phone: &str,
        body: &str,
    ) -> Result<(), EngagementError> {
        self.broadcast.send_whatsapp(phone, body).await
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EngagementError> {
        self.broadcast.send_email(to, subject, body).await
    }

    pub async fn broadcast(
        &self,
        input: &BroadcastInput,
        contacts: &[(String, Option<String>)], // (identifier, name)
    ) -> Result<u32, EngagementError> {
        let mut sent: u32 = 0;
        for (contact, _name) in contacts {
            let result = match input.channel.as_str() {
                "whatsapp" => self.broadcast.send_whatsapp(contact, &input.body).await,
                "email" => {
                    let subject = input.subject.as_deref().unwrap_or("Message from your therapist");
                    self.broadcast.send_email(contact, subject, &input.body).await
                }
                _ => Err(EngagementError::BroadcastFailed(format!(
                    "Unknown channel: {}",
                    input.channel
                ))),
            };
            if result.is_ok() {
                sent += 1;
            }
        }
        Ok(sent)
    }
}
