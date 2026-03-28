use async_trait::async_trait;
use uuid::Uuid;

use super::entity::{Invoice, PaymentOrder};
use super::error::BillingError;

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Invoice>, BillingError>;

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Invoice>, i64), BillingError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        session_id: Option<Uuid>,
        invoice_number: &str,
        amount_inr: i32,
        gst_amount_inr: i32,
        total_inr: i32,
    ) -> Result<Invoice, BillingError>;

    async fn mark_paid(
        &self,
        id: Uuid,
        razorpay_payment_id: &str,
        razorpay_order_id: Option<&str>,
    ) -> Result<Invoice, BillingError>;

    async fn next_sequence_number(
        &self,
        therapist_id: Uuid,
        year_month: &str,
    ) -> Result<i32, BillingError>;
}

#[async_trait]
pub trait PaymentGatewayPort: Send + Sync {
    async fn create_order(
        &self,
        amount_paise: i64,
        invoice_id: Uuid,
    ) -> Result<PaymentOrder, BillingError>;
}
