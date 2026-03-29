use async_trait::async_trait;
use uuid::Uuid;

use super::entity::*;
use super::error::EngagementError;

// ─── Resource Repository ────────────────────────────────────────────────────

#[async_trait]
pub trait ResourceRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<Resource>, EngagementError>;

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Resource>, i64), EngagementError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateResourceInput,
    ) -> Result<Resource, EngagementError>;

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateResourceInput,
    ) -> Result<Resource, EngagementError>;

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), EngagementError>;

    async fn share(
        &self,
        resource_id: Uuid,
        therapist_id: Uuid,
        client_ids: &[Uuid],
        note: Option<&str>,
    ) -> Result<Vec<ClientResource>, EngagementError>;

    async fn unshare(
        &self,
        resource_id: Uuid,
        therapist_id: Uuid,
        client_ids: &[Uuid],
    ) -> Result<(), EngagementError>;

    async fn list_shared_with_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ClientResource>, i64), EngagementError>;
}

// ─── Intake Form Repository ────────────────────────────────────────────────

#[async_trait]
pub trait IntakeFormRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<IntakeForm>, EngagementError>;

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IntakeForm>, i64), EngagementError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeFormInput,
    ) -> Result<IntakeForm, EngagementError>;

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateIntakeFormInput,
    ) -> Result<IntakeForm, EngagementError>;

    async fn delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), EngagementError>;

    // ─── Responses ──────────────────────────────────────────────────────

    async fn create_response(
        &self,
        therapist_id: Uuid,
        input: &CreateIntakeResponseInput,
        form_snapshot: &serde_json::Value,
    ) -> Result<IntakeResponse, EngagementError>;

    async fn find_response_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<IntakeResponse>, EngagementError>;

    async fn find_response_by_token(
        &self,
        token: Uuid,
    ) -> Result<Option<IntakeResponse>, EngagementError>;

    async fn list_responses_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IntakeResponse>, i64), EngagementError>;

    async fn submit_response(
        &self,
        id: Uuid,
        encrypted_responses: &str,
    ) -> Result<IntakeResponse, EngagementError>;
}

// ─── Broadcast Port ─────────────────────────────────────────────────────────

#[async_trait]
pub trait BroadcastPort: Send + Sync {
    async fn send_whatsapp(
        &self,
        phone: &str,
        body: &str,
    ) -> Result<(), EngagementError>;

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EngagementError>;
}

// ─── Encryption Port ────────────────────────────────────────────────────────

pub trait EngagementEncryptionPort: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> Result<String, EngagementError>;
    fn decrypt(&self, ciphertext: &str) -> Result<String, EngagementError>;
}
