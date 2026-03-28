use std::sync::Arc;
use sqlx::PgPool;

use crate::shared::config::AppConfig;
use crate::shared::encryption::EncryptionService;

// ─── IAM ─────────────────────────────────────────────────────────────────────
use crate::iam::application::service::{TherapistService, PracticeService, OnboardingService};
use crate::iam::infra::therapist_repo::PgTherapistRepository;
use crate::iam::infra::availability_repo::PgAvailabilityRepository;
use crate::iam::infra::practice_repo::PgPracticeRepository;
use crate::iam::infra::invitation_repo::PgInvitationRepository;
use crate::iam::infra::onboarding_repo::PgOnboardingTokenRepository;

// ─── Scheduling ──────────────────────────────────────────────────────────────
use crate::scheduling::application::service::{
    SessionService, BookingService, BlockedSlotService,
    RecurringReservationService, SessionTypeService,
};
use crate::scheduling::infra::session_repo::PgSessionRepository;
use crate::scheduling::infra::blocked_slot_repo::PgBlockedSlotRepository;
use crate::scheduling::infra::recurring_repo::PgRecurringReservationRepository;
use crate::scheduling::infra::session_type_repo::PgSessionTypeRepository;

// ─── Clients ─────────────────────────────────────────────────────────────────
use crate::clients::application::service::{ClientService, ClientPortalService};
use crate::clients::infra::client_repo::PgClientRepository;
use crate::clients::infra::portal_repo::PgClientPortalRepository;

// ─── Clinical ────────────────────────────────────────────────────────────────
use crate::clinical::application::service::{NoteService, TreatmentPlanService, MessageService};
use crate::clinical::infra::note_repo::PgNoteRepository;
use crate::clinical::infra::treatment_plan_repo::PgTreatmentPlanRepository;
use crate::clinical::infra::message_repo::PgMessageRepository;
use crate::clinical::infra::encryption_adapter::EncryptionAdapter as ClinicalEncryptionAdapter;

// ─── Billing ─────────────────────────────────────────────────────────────────
use crate::billing::application::service::PaymentService;
use crate::billing::infra::invoice_repo::PgInvoiceRepository;
use crate::billing::infra::razorpay_gateway::RazorpayGateway;

// ─── Engagement ──────────────────────────────────────────────────────────────
use crate::engagement::application::service::{ResourceService, IntakeService, BroadcastService};
use crate::engagement::infra::resource_repo::PgResourceRepository;
use crate::engagement::infra::intake_form_repo::PgIntakeFormRepository;
use crate::engagement::infra::broadcast_adapter::HttpBroadcastAdapter;
use crate::engagement::infra::encryption_adapter::EngagementEncryptionAdapter;

// ─── Analytics ───────────────────────────────────────────────────────────────
use crate::analytics::application::service::AnalyticsService;
use crate::analytics::infra::analytics_repo::PgAnalyticsRepository;

/// All wired services, ready to be injected into Actix `app_data`.
pub struct AppServices {
    // IAM
    pub therapist_service: TherapistService,
    pub practice_service: PracticeService,
    pub onboarding_service: OnboardingService,

    // Scheduling
    pub session_service: SessionService,
    pub booking_service: BookingService,
    pub blocked_slot_service: BlockedSlotService,
    pub recurring_service: RecurringReservationService,
    pub session_type_service: SessionTypeService,

    // Clients
    pub client_service: ClientService,
    pub client_portal_service: ClientPortalService,

    // Clinical
    pub note_service: NoteService,
    pub treatment_plan_service: TreatmentPlanService,
    pub message_service: MessageService,

    // Billing
    pub payment_service: PaymentService,

    // Engagement
    pub resource_service: ResourceService,
    pub intake_service: IntakeService,
    pub broadcast_service: BroadcastService,

    // Analytics
    pub analytics_service: AnalyticsService,
}

impl AppServices {
    /// Wire everything with Arc<dyn Trait>.
    /// This is the composition root — the ONLY place that knows about concrete types.
    pub fn new(pool: PgPool, config: &AppConfig) -> Self {
        let encryption = EncryptionService::from_base64_key(&config.encryption_key)
            .expect("Invalid ENCRYPTION_KEY");

        // ── IAM ──────────────────────────────────────────────────────────
        let therapist_repo = Arc::new(PgTherapistRepository::new(pool.clone()));
        let availability_repo = Arc::new(PgAvailabilityRepository::new(pool.clone()));
        let practice_repo = Arc::new(PgPracticeRepository::new(pool.clone()));
        let invitation_repo = Arc::new(PgInvitationRepository::new(pool.clone()));
        let onboarding_repo = Arc::new(PgOnboardingTokenRepository::new(pool.clone()));

        let therapist_service = TherapistService::new(
            therapist_repo.clone(),
            availability_repo.clone(),
        );
        let practice_service = PracticeService::new(
            practice_repo.clone(),
            invitation_repo.clone(),
        );
        let onboarding_service = OnboardingService::new(onboarding_repo.clone());

        // ── Scheduling ───────────────────────────────────────────────────
        let session_repo = Arc::new(PgSessionRepository::new(pool.clone()));
        let blocked_slot_repo = Arc::new(PgBlockedSlotRepository::new(pool.clone()));
        let recurring_repo = Arc::new(PgRecurringReservationRepository::new(pool.clone()));
        let session_type_repo = Arc::new(PgSessionTypeRepository::new(pool.clone()));

        let session_service = SessionService::new(session_repo.clone());
        let booking_service = BookingService::new(
            session_repo.clone(),
            blocked_slot_repo.clone(),
            recurring_repo.clone(),
        );
        let blocked_slot_service = BlockedSlotService::new(blocked_slot_repo.clone());
        let recurring_service = RecurringReservationService::new(
            recurring_repo.clone(),
            session_repo.clone(),
        );
        let session_type_service = SessionTypeService::new(session_type_repo.clone());

        // ── Clients ──────────────────────────────────────────────────────
        let client_repo = Arc::new(PgClientRepository::new(pool.clone()));
        let portal_repo = Arc::new(PgClientPortalRepository::new(pool.clone()));

        let client_service = ClientService::new(client_repo.clone());
        let client_portal_service = ClientPortalService::new(portal_repo.clone());

        // ── Clinical ─────────────────────────────────────────────────────
        let note_repo = Arc::new(PgNoteRepository::new(pool.clone()));
        let plan_repo = Arc::new(PgTreatmentPlanRepository::new(pool.clone()));
        let message_repo = Arc::new(PgMessageRepository::new(pool.clone()));
        let clinical_encryption: Arc<dyn crate::clinical::domain::port::EncryptionPort> =
            Arc::new(ClinicalEncryptionAdapter::new(encryption.clone()));

        let note_service = NoteService::new(note_repo.clone(), clinical_encryption.clone());
        let treatment_plan_service = TreatmentPlanService::new(plan_repo.clone(), clinical_encryption.clone());
        let message_service = MessageService::new(message_repo.clone(), clinical_encryption.clone());

        // ── Billing ──────────────────────────────────────────────────────
        let invoice_repo = Arc::new(PgInvoiceRepository::new(pool.clone()));
        let payment_gateway: Arc<dyn crate::billing::domain::port::PaymentGatewayPort> =
            Arc::new(RazorpayGateway::new(
                config.razorpay_key_id.clone().unwrap_or_default(),
                config.razorpay_key_secret.clone().unwrap_or_default(),
            ));

        let payment_service = PaymentService::new(invoice_repo.clone(), payment_gateway);

        // ── Engagement ───────────────────────────────────────────────────
        let resource_repo = Arc::new(PgResourceRepository::new(pool.clone()));
        let intake_repo = Arc::new(PgIntakeFormRepository::new(pool.clone()));
        let broadcast_port: Arc<dyn crate::engagement::domain::port::BroadcastPort> =
            Arc::new(HttpBroadcastAdapter::new(
                config.gupshup_api_key.clone().unwrap_or_default(),
                config.gupshup_app_name.clone().unwrap_or_default(),
                config.resend_api_key.clone().unwrap_or_default(),
                "noreply@bendre.app".to_string(),
            ));
        let engagement_encryption: Arc<dyn crate::engagement::domain::port::EngagementEncryptionPort> =
            Arc::new(EngagementEncryptionAdapter::new(encryption.clone()));

        let resource_service = ResourceService::new(resource_repo.clone());
        let intake_service = IntakeService::new(intake_repo.clone(), engagement_encryption);
        let broadcast_service = BroadcastService::new(broadcast_port);

        // ── Analytics ────────────────────────────────────────────────────
        let analytics_repo = Arc::new(PgAnalyticsRepository::new(pool.clone()));
        let analytics_service = AnalyticsService::new(analytics_repo);

        Self {
            therapist_service,
            practice_service,
            onboarding_service,
            session_service,
            booking_service,
            blocked_slot_service,
            recurring_service,
            session_type_service,
            client_service,
            client_portal_service,
            note_service,
            treatment_plan_service,
            message_service,
            payment_service,
            resource_service,
            intake_service,
            broadcast_service,
            analytics_service,
        }
    }
}
