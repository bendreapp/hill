use std::sync::Arc;
use uuid::Uuid;

use crate::iam::domain::entity::*;
use crate::iam::domain::error::IamError;
use crate::iam::domain::port::*;
use crate::leads::domain::entity::{CreateLeadInput};
use crate::leads::domain::port::LeadRepository;

// ─── Therapist Service ───────────────────────────────────────────────────────

pub struct TherapistService {
    pub therapist_repo: Arc<dyn TherapistRepository>,
    pub availability_repo: Arc<dyn AvailabilityRepository>,
    pub lead_repo: Arc<dyn LeadRepository>,
}

impl TherapistService {
    pub fn new(
        therapist_repo: Arc<dyn TherapistRepository>,
        availability_repo: Arc<dyn AvailabilityRepository>,
        lead_repo: Arc<dyn LeadRepository>,
    ) -> Self {
        Self {
            therapist_repo,
            availability_repo,
            lead_repo,
        }
    }

    pub async fn get_me(&self, user_id: Uuid) -> Result<Therapist, IamError> {
        self.therapist_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IamError::TherapistNotFound)
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Therapist, IamError> {
        self.therapist_repo
            .find_by_slug(slug)
            .await?
            .ok_or(IamError::TherapistNotFound)
    }

    pub async fn update(&self, therapist: &Therapist) -> Result<Therapist, IamError> {
        // Validate slug uniqueness if changed
        if self
            .therapist_repo
            .slug_exists(&therapist.slug, Some(therapist.id))
            .await?
        {
            return Err(IamError::SlugTaken);
        }
        self.therapist_repo.update(therapist).await
    }

    /// Phase 1: therapist selects a plan (solo/team/clinic/organization).
    /// Sets plan_selected + plan_status = "pending" and creates a lead entry for HQ.
    pub async fn select_plan(
        &self,
        user_id: Uuid,
        plan: &str,
        email: Option<String>,
    ) -> Result<Therapist, IamError> {
        // Validate plan value
        let valid_plans = ["solo", "team", "clinic", "organization"];
        if !valid_plans.contains(&plan) {
            return Err(IamError::InvalidPlan);
        }

        let mut therapist = self
            .therapist_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IamError::TherapistNotFound)?;

        therapist.plan_selected = Some(plan.to_string());
        therapist.plan_status = "pending".to_string();

        let updated = self.therapist_repo.update(&therapist).await?;

        // Create a lead entry in Bendre HQ so the team can see this signup
        let lead_input = CreateLeadInput {
            full_name: updated.full_name.clone(),
            email,
            phone: updated.phone.clone(),
            reason: None,
            source: Some("signup".to_string()),
            preferred_times: None,
            message: Some(format!("Plan selected: {}", plan)),
        };
        // Best-effort — don't fail the whole request if lead creation fails
        let _ = self.lead_repo.create(user_id, &lead_input).await;

        Ok(updated)
    }

    /// Phase 1: therapist completes the onboarding wizard.
    /// Sets avatar_key, bio, support_requested, and onboarding_complete = true.
    pub async fn complete_onboarding(
        &self,
        user_id: Uuid,
        avatar_key: Option<String>,
        bio: Option<String>,
        support_requested: bool,
    ) -> Result<Therapist, IamError> {
        let mut therapist = self
            .therapist_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IamError::TherapistNotFound)?;

        if let Some(k) = avatar_key {
            therapist.avatar_key = Some(k);
        }
        if let Some(b) = bio {
            therapist.bio = Some(b);
        }
        therapist.support_requested = support_requested;
        therapist.onboarding_complete = true;

        self.therapist_repo.update(&therapist).await
    }

    pub async fn get_availability(&self, therapist_id: Uuid) -> Result<Vec<Availability>, IamError> {
        self.availability_repo
            .find_by_therapist(therapist_id)
            .await
    }

    pub async fn set_availability(
        &self,
        therapist_id: Uuid,
        day_of_week: i16,
        start_time: chrono::NaiveTime,
        end_time: chrono::NaiveTime,
        is_active: bool,
    ) -> Result<Availability, IamError> {
        self.availability_repo
            .upsert(therapist_id, day_of_week, start_time, end_time, is_active)
            .await
    }
}

// ─── Practice Service ────────────────────────────────────────────────────────

pub struct PracticeService {
    pub practice_repo: Arc<dyn PracticeRepository>,
    pub invitation_repo: Arc<dyn InvitationRepository>,
}

impl PracticeService {
    pub fn new(
        practice_repo: Arc<dyn PracticeRepository>,
        invitation_repo: Arc<dyn InvitationRepository>,
    ) -> Self {
        Self {
            practice_repo,
            invitation_repo,
        }
    }

    pub async fn get_my_practice(&self, user_id: Uuid) -> Result<Option<(Practice, PracticeMember)>, IamError> {
        self.practice_repo.find_by_member(user_id).await
    }

    pub async fn create_practice(&self, name: &str, owner_id: Uuid) -> Result<Practice, IamError> {
        // Check if user already has a practice
        if self.practice_repo.find_by_owner(owner_id).await?.is_some() {
            return Err(IamError::AlreadyHasPractice);
        }
        let practice = self.practice_repo.create(name, owner_id).await?;

        // Auto-add owner as member
        self.practice_repo
            .add_member(practice.id, owner_id, Some(owner_id), "owner", true)
            .await?;

        Ok(practice)
    }

    pub async fn list_members(&self, practice_id: Uuid) -> Result<Vec<PracticeMember>, IamError> {
        self.practice_repo.list_members(practice_id).await
    }

    pub async fn update_member(
        &self,
        caller_id: Uuid,
        practice_id: Uuid,
        member_id: Uuid,
        role: &str,
        can_view_notes: bool,
    ) -> Result<PracticeMember, IamError> {
        self.ensure_owner(caller_id, practice_id).await?;
        self.practice_repo
            .update_member(member_id, role, can_view_notes)
            .await
    }

    pub async fn remove_member(
        &self,
        caller_id: Uuid,
        practice_id: Uuid,
        member_id: Uuid,
    ) -> Result<(), IamError> {
        self.ensure_owner(caller_id, practice_id).await?;

        // Cannot remove the owner
        let practice = self
            .practice_repo
            .find_by_id(practice_id)
            .await?
            .ok_or(IamError::PracticeNotFound)?;

        let member = self
            .practice_repo
            .find_member(practice_id, member_id)
            .await?
            .ok_or(IamError::MemberNotFound)?;

        if member.user_id == practice.owner_id {
            return Err(IamError::CannotRemoveOwner);
        }

        self.practice_repo.remove_member(member_id).await
    }

    pub async fn create_invitation(
        &self,
        caller_id: Uuid,
        practice_id: Uuid,
        email: Option<&str>,
        role: &str,
        can_view_notes: bool,
    ) -> Result<PracticeInvitation, IamError> {
        self.ensure_owner(caller_id, practice_id).await?;
        self.invitation_repo
            .create(practice_id, caller_id, email, role, can_view_notes)
            .await
    }

    pub async fn list_invitations(&self, practice_id: Uuid) -> Result<Vec<PracticeInvitation>, IamError> {
        self.invitation_repo.list_by_practice(practice_id).await
    }

    pub async fn get_invitation_by_token(&self, token: Uuid) -> Result<PracticeInvitation, IamError> {
        self.invitation_repo
            .find_by_token(token)
            .await?
            .ok_or(IamError::InvitationNotFound)
    }

    pub async fn accept_invitation(
        &self,
        token: Uuid,
        user_id: Uuid,
    ) -> Result<PracticeInvitation, IamError> {
        let invitation = self
            .invitation_repo
            .find_by_token(token)
            .await?
            .ok_or(IamError::InvitationNotFound)?;

        if invitation.status != "pending" {
            return Err(IamError::InvitationAlreadyAccepted);
        }

        if invitation.expires_at < chrono::Utc::now() {
            return Err(IamError::InvitationExpired);
        }

        // Add user as practice member
        self.practice_repo
            .add_member(
                invitation.practice_id,
                user_id,
                Some(user_id),
                &invitation.role,
                invitation.can_view_notes,
            )
            .await?;

        // Mark invitation as accepted
        self.invitation_repo.accept(invitation.id, user_id).await
    }

    pub async fn revoke_invitation(
        &self,
        caller_id: Uuid,
        practice_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), IamError> {
        self.ensure_owner(caller_id, practice_id).await?;
        self.invitation_repo.revoke(invitation_id).await
    }

    async fn ensure_owner(&self, user_id: Uuid, practice_id: Uuid) -> Result<(), IamError> {
        let practice = self
            .practice_repo
            .find_by_id(practice_id)
            .await?
            .ok_or(IamError::PracticeNotFound)?;

        if practice.owner_id != user_id {
            return Err(IamError::NotPracticeOwner);
        }
        Ok(())
    }
}

// ─── Onboarding Service ──────────────────────────────────────────────────────

pub struct OnboardingService {
    pub token_repo: Arc<dyn OnboardingTokenRepository>,
}

impl OnboardingService {
    pub fn new(token_repo: Arc<dyn OnboardingTokenRepository>) -> Self {
        Self { token_repo }
    }

    pub async fn create_token(
        &self,
        therapist_id: Uuid,
        label: Option<&str>,
        max_uses: Option<i32>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OnboardingToken, IamError> {
        self.token_repo
            .create(therapist_id, label, max_uses, expires_at)
            .await
    }

    pub async fn list_tokens(&self, therapist_id: Uuid) -> Result<Vec<OnboardingToken>, IamError> {
        self.token_repo.list_by_therapist(therapist_id).await
    }

    pub async fn toggle_token(&self, id: Uuid, is_active: bool) -> Result<OnboardingToken, IamError> {
        self.token_repo.toggle_active(id, is_active).await
    }

    pub async fn validate_token(&self, token: Uuid) -> Result<OnboardingToken, IamError> {
        let t = self
            .token_repo
            .find_by_token(token)
            .await?
            .ok_or(IamError::OnboardingTokenNotFound)?;

        if !t.is_active {
            return Err(IamError::OnboardingTokenInvalid);
        }
        if let Some(max) = t.max_uses {
            if t.use_count >= max {
                return Err(IamError::OnboardingTokenInvalid);
            }
        }
        if let Some(exp) = t.expires_at {
            if exp < chrono::Utc::now() {
                return Err(IamError::OnboardingTokenInvalid);
            }
        }

        Ok(t)
    }

    pub async fn consume_token(&self, token: Uuid) -> Result<OnboardingToken, IamError> {
        let t = self.validate_token(token).await?;
        self.token_repo.increment_use_count(t.id).await?;
        Ok(t)
    }
}
