use std::sync::Arc;
use uuid::Uuid;

use crate::engagement::domain::entity::*;
use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::*;
use crate::shared::email::{send_email, EmailParams};

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

// ─── Lead Intake Service ─────────────────────────────────────────────────────

pub struct LeadIntakeService {
    pub submission_repo: Arc<dyn crate::engagement::domain::port::LeadIntakeSubmissionRepository>,
    pub question_repo: Arc<dyn crate::engagement::domain::port::IntakeFormQuestionRepository>,
    pub resend_api_key: String,
    pub frontend_url: String,
}

impl LeadIntakeService {
    pub fn new(
        submission_repo: Arc<dyn crate::engagement::domain::port::LeadIntakeSubmissionRepository>,
        question_repo: Arc<dyn crate::engagement::domain::port::IntakeFormQuestionRepository>,
        resend_api_key: String,
        frontend_url: String,
    ) -> Self {
        Self {
            submission_repo,
            question_repo,
            resend_api_key,
            frontend_url,
        }
    }

    /// Send an intake form to a lead. Creates a submission record and sends email.
    /// Also updates lead status to `contacted`.
    pub async fn send_to_lead(
        &self,
        lead_id: Uuid,
        therapist_id: Uuid,
        lead_name: &str,
        lead_email: &str,
        therapist_display_name: &str,
        template_subject: &str,
        template_body: &str,
    ) -> Result<SendLeadIntakeResponse, EngagementError> {
        // Generate unique access token
        let access_token = Uuid::new_v4().to_string();

        // Create submission record
        let submission = self.submission_repo
            .create(lead_id, therapist_id, &access_token)
            .await?;

        // Build intake link
        let intake_link = format!("{}/intake/submit/{}", self.frontend_url, access_token);

        // Replace template variables
        let subject = template_subject
            .replace("{client_name}", lead_name)
            .replace("{therapist_name}", therapist_display_name)
            .replace("{intake_link}", &intake_link);

        let body_text = template_body
            .replace("{client_name}", lead_name)
            .replace("{therapist_name}", therapist_display_name)
            .replace("{intake_link}", &format!(
                "<a href=\"{}\" style=\"background:#5C7A6B;color:white;padding:10px 20px;border-radius:6px;text-decoration:none;display:inline-block\">Fill out intake form</a>",
                intake_link
            ));

        let html = format!(
            "<p>Hi {},</p><p>{}</p><p>— {} via Bendre</p>",
            lead_name, body_text, therapist_display_name
        );

        let params = EmailParams {
            to: lead_email,
            reply_to: None,
            from_name: &format!("{} via Bendre", therapist_display_name),
            subject: &subject,
            html: &html,
        };

        if let Err(e) = send_email(&self.resend_api_key, params).await {
            tracing::warn!("Failed to send intake form email to lead {}: {}", lead_id, e);
            // Return error — this is a deliberate user action, not a best-effort notification
            return Err(EngagementError::BroadcastFailed(e));
        }

        Ok(SendLeadIntakeResponse {
            submission_id: submission.id,
            access_token: submission.access_token,
            sent_at: submission.sent_at,
        })
    }

    /// Public: get the intake form for a lead by access token.
    /// Returns 410 Gone (via error) if already submitted.
    pub async fn get_public_form(
        &self,
        token: &str,
    ) -> Result<PublicLeadIntakeFormData, EngagementError> {
        let submission = self.submission_repo
            .find_by_token(token)
            .await?
            .ok_or(EngagementError::IntakeResponseNotFound)?;

        if submission.submitted_at.is_some() {
            return Err(EngagementError::AlreadySubmitted);
        }

        // Fetch questions for this therapist
        let questions = self.question_repo
            .list_by_therapist(submission.therapist_id)
            .await?;

        Ok(PublicLeadIntakeFormData {
            submission_id: submission.id,
            therapist_id: submission.therapist_id,
            lead_id: submission.lead_id,
            questions,
        })
    }

    /// Public: submit intake form responses.
    pub async fn submit_public_form(
        &self,
        token: &str,
        responses: &serde_json::Value,
    ) -> Result<LeadIntakeSubmission, EngagementError> {
        // Verify token and check not already submitted
        let submission = self.submission_repo
            .find_by_token(token)
            .await?
            .ok_or(EngagementError::IntakeResponseNotFound)?;

        if submission.submitted_at.is_some() {
            return Err(EngagementError::AlreadySubmitted);
        }

        self.submission_repo.submit(token, responses).await
    }

    /// Authenticated: list all intake submissions for a lead.
    pub async fn list_by_lead(
        &self,
        lead_id: Uuid,
        therapist_id: Uuid,
    ) -> Result<Vec<LeadIntakeSubmission>, EngagementError> {
        self.submission_repo.list_by_lead(lead_id, therapist_id).await
    }
}

/// Intermediate struct for building the public form response.
pub struct PublicLeadIntakeFormData {
    pub submission_id: Uuid,
    pub therapist_id: Uuid,
    pub lead_id: Uuid,
    pub questions: Vec<IntakeFormQuestion>,
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
