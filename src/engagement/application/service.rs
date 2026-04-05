use std::sync::Arc;
use chrono::Utc;
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

// ─── Message Template Service ────────────────────────────────────────────────

pub struct MessageTemplateService {
    pub template_repo: Arc<dyn crate::engagement::domain::port::MessageTemplateRepository>,
}

/// Default templates returned when no custom template has been saved.
struct DefaultTemplate {
    key: &'static str,
    subject: &'static str,
    body: &'static str,
}

const DEFAULTS: &[DefaultTemplate] = &[
    DefaultTemplate {
        key: "inquiry_ack",
        subject: "Thanks for reaching out",
        body: "Hi {client_name}, thanks for reaching out. I'll review your message and get back to you shortly.",
    },
    DefaultTemplate {
        key: "intake_invite",
        subject: "Please fill out my intake form",
        body: "Hi {client_name}, please fill out this intake form so I can understand your needs better: {intake_link}",
    },
    DefaultTemplate {
        key: "portal_invite",
        subject: "Welcome — set up your client portal",
        body: "Hi {client_name}, welcome aboard! Set up your client portal here: {portal_link}",
    },
    DefaultTemplate {
        key: "rejection",
        subject: "Re: Your inquiry",
        body: "Hi {client_name}, thank you for reaching out. After reviewing your inquiry, I don't think I'm the right fit for your needs at this time. I wish you the best.",
    },
];

const VALID_KEYS: &[&str] = &["inquiry_ack", "intake_invite", "portal_invite", "rejection"];

impl MessageTemplateService {
    pub fn new(template_repo: Arc<dyn crate::engagement::domain::port::MessageTemplateRepository>) -> Self {
        Self { template_repo }
    }

    /// Returns all 4 templates for the therapist.
    /// For any key that doesn't have a custom template yet, returns the default.
    pub async fn list_templates(&self, therapist_id: Uuid) -> Result<Vec<MessageTemplate>, EngagementError> {
        let saved = self.template_repo.find_all_by_therapist(therapist_id).await?;

        let templates = DEFAULTS
            .iter()
            .map(|default| {
                // If the therapist has customized this key, use their version
                saved
                    .iter()
                    .find(|t| t.template_key == default.key)
                    .cloned()
                    .unwrap_or_else(|| MessageTemplate {
                        id: uuid::Uuid::nil(),
                        therapist_id,
                        template_key: default.key.to_string(),
                        subject: default.subject.to_string(),
                        body: default.body.to_string(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    })
            })
            .collect();

        Ok(templates)
    }

    /// Saves (upserts) a custom template for the therapist.
    pub async fn update_template(
        &self,
        therapist_id: Uuid,
        key: &str,
        subject: &str,
        body: &str,
    ) -> Result<MessageTemplate, EngagementError> {
        if !VALID_KEYS.contains(&key) {
            return Err(EngagementError::InvalidTemplateKey(format!(
                "'{}' is not a valid template key",
                key
            )));
        }

        self.template_repo
            .upsert(therapist_id, key, subject, body)
            .await
    }

    /// Returns the template (custom or default) for a specific key — for use when sending emails.
    pub async fn get_template_for_sending(
        &self,
        therapist_id: Uuid,
        key: &str,
    ) -> Result<MessageTemplate, EngagementError> {
        if !VALID_KEYS.contains(&key) {
            return Err(EngagementError::InvalidTemplateKey(format!(
                "'{}' is not a valid template key",
                key
            )));
        }

        let saved = self.template_repo.find_by_key(therapist_id, key).await?;

        if let Some(template) = saved {
            return Ok(template);
        }

        // Return default
        let default = DEFAULTS.iter().find(|d| d.key == key).unwrap();
        Ok(MessageTemplate {
            id: uuid::Uuid::nil(),
            therapist_id,
            template_key: default.key.to_string(),
            subject: default.subject.to_string(),
            body: default.body.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

// ─── Intake Question Service ─────────────────────────────────────────────────

pub struct IntakeQuestionService {
    pub question_repo: Arc<dyn crate::engagement::domain::port::IntakeFormQuestionRepository>,
}

impl IntakeQuestionService {
    pub fn new(
        question_repo: Arc<dyn crate::engagement::domain::port::IntakeFormQuestionRepository>,
    ) -> Self {
        Self { question_repo }
    }

    /// Returns the therapist's questions. Seeds defaults if none exist yet.
    pub async fn list_questions(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<IntakeFormQuestion>, EngagementError> {
        let existing = self.question_repo.list_by_therapist(therapist_id).await?;
        if existing.is_empty() {
            return self.question_repo.seed_defaults(therapist_id).await;
        }
        Ok(existing)
    }

    pub async fn create_question(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeQuestionInput,
    ) -> Result<IntakeFormQuestion, EngagementError> {
        self.question_repo.create(therapist_id, input).await
    }

    pub async fn update_question(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateIntakeQuestionInput,
    ) -> Result<IntakeFormQuestion, EngagementError> {
        self.question_repo.update(id, therapist_id, input).await
    }

    pub async fn delete_question(
        &self,
        id: Uuid,
        therapist_id: Uuid,
    ) -> Result<(), EngagementError> {
        self.question_repo.delete(id, therapist_id).await
    }

    pub async fn reorder_questions(
        &self,
        therapist_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<IntakeFormQuestion>, EngagementError> {
        self.question_repo.reorder(therapist_id, ids).await
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
