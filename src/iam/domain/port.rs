use async_trait::async_trait;
use chrono::NaiveTime;
use uuid::Uuid;

use super::entity::*;
use super::error::IamError;

// ─── Therapist Repository ────────────────────────────────────────────────────

#[async_trait]
pub trait TherapistRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Therapist>, IamError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Therapist>, IamError>;
    async fn update(&self, therapist: &Therapist) -> Result<Therapist, IamError>;
    async fn slug_exists(&self, slug: &str, exclude_id: Option<Uuid>) -> Result<bool, IamError>;
}

// ─── Availability Repository ─────────────────────────────────────────────────

#[async_trait]
pub trait AvailabilityRepository: Send + Sync {
    async fn find_by_therapist(&self, therapist_id: Uuid) -> Result<Vec<Availability>, IamError>;
    async fn upsert(
        &self,
        therapist_id: Uuid,
        day_of_week: i16,
        start_time: NaiveTime,
        end_time: NaiveTime,
        is_active: bool,
    ) -> Result<Availability, IamError>;
}

// ─── Practice Repository ─────────────────────────────────────────────────────

#[async_trait]
pub trait PracticeRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Practice>, IamError>;
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Option<Practice>, IamError>;
    async fn find_by_member(&self, user_id: Uuid) -> Result<Option<(Practice, PracticeMember)>, IamError>;
    async fn create(&self, name: &str, owner_id: Uuid) -> Result<Practice, IamError>;

    async fn list_members(&self, practice_id: Uuid) -> Result<Vec<PracticeMember>, IamError>;
    async fn find_member(&self, practice_id: Uuid, user_id: Uuid) -> Result<Option<PracticeMember>, IamError>;
    async fn update_member(&self, member_id: Uuid, role: &str, can_view_notes: bool) -> Result<PracticeMember, IamError>;
    async fn remove_member(&self, member_id: Uuid) -> Result<(), IamError>;
    async fn add_member(
        &self,
        practice_id: Uuid,
        user_id: Uuid,
        therapist_id: Option<Uuid>,
        role: &str,
        can_view_notes: bool,
    ) -> Result<PracticeMember, IamError>;

    async fn get_accessible_therapist_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, IamError>;
}

// ─── Invitation Repository ───────────────────────────────────────────────────

#[async_trait]
pub trait InvitationRepository: Send + Sync {
    async fn create(
        &self,
        practice_id: Uuid,
        invited_by: Uuid,
        email: Option<&str>,
        role: &str,
        can_view_notes: bool,
    ) -> Result<PracticeInvitation, IamError>;
    async fn find_by_token(&self, token: Uuid) -> Result<Option<PracticeInvitation>, IamError>;
    async fn list_by_practice(&self, practice_id: Uuid) -> Result<Vec<PracticeInvitation>, IamError>;
    async fn accept(&self, id: Uuid, accepted_by: Uuid) -> Result<PracticeInvitation, IamError>;
    async fn revoke(&self, id: Uuid) -> Result<(), IamError>;
}

// ─── Onboarding Token Repository ─────────────────────────────────────────────

#[async_trait]
pub trait OnboardingTokenRepository: Send + Sync {
    async fn create(
        &self,
        therapist_id: Uuid,
        label: Option<&str>,
        max_uses: Option<i32>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OnboardingToken, IamError>;
    async fn find_by_token(&self, token: Uuid) -> Result<Option<OnboardingToken>, IamError>;
    async fn list_by_therapist(&self, therapist_id: Uuid) -> Result<Vec<OnboardingToken>, IamError>;
    async fn toggle_active(&self, id: Uuid, is_active: bool) -> Result<OnboardingToken, IamError>;
    async fn increment_use_count(&self, id: Uuid) -> Result<(), IamError>;
}
