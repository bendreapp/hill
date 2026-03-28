use std::sync::Arc;
use uuid::Uuid;

use crate::billing::domain::entity::*;
use crate::billing::domain::error::BillingError;
use crate::billing::domain::port::*;

pub struct PaymentService {
    pub invoice_repo: Arc<dyn InvoiceRepository>,
    pub gateway: Arc<dyn PaymentGatewayPort>,
}

impl PaymentService {
    pub fn new(
        invoice_repo: Arc<dyn InvoiceRepository>,
        gateway: Arc<dyn PaymentGatewayPort>,
    ) -> Self {
        Self {
            invoice_repo,
            gateway,
        }
    }

    pub async fn get_invoice(&self, id: Uuid) -> Result<Invoice, BillingError> {
        self.invoice_repo
            .find_by_id(id)
            .await?
            .ok_or(BillingError::InvoiceNotFound)
    }

    pub async fn list_invoices(
        &self,
        therapist_ids: &[Uuid],
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Invoice>, i64), BillingError> {
        self.invoice_repo
            .list(therapist_ids, status, limit, offset)
            .await
    }

    pub async fn create_invoice(
        &self,
        therapist_id: Uuid,
        input: &CreateInvoiceInput,
    ) -> Result<Invoice, BillingError> {
        // Generate invoice number: MANO-YYYYMM-XXXX
        let now = chrono::Utc::now();
        let year_month = now.format("%Y%m").to_string();
        let seq = self
            .invoice_repo
            .next_sequence_number(therapist_id, &year_month)
            .await?;
        let invoice_number = format!("MANO-{}-{:04}", year_month, seq);

        // Calculate GST
        let gst_percent = input.gst_percent.unwrap_or(0);
        let gst_amount_inr = (input.amount_inr as i64 * gst_percent as i64 / 100) as i32;
        let total_inr = input.amount_inr + gst_amount_inr;

        self.invoice_repo
            .create(
                therapist_id,
                input.client_id,
                input.session_id,
                &invoice_number,
                input.amount_inr,
                gst_amount_inr,
                total_inr,
            )
            .await
    }

    pub async fn mark_paid(
        &self,
        id: Uuid,
        razorpay_payment_id: &str,
        razorpay_order_id: Option<&str>,
    ) -> Result<Invoice, BillingError> {
        self.invoice_repo
            .find_by_id(id)
            .await?
            .ok_or(BillingError::InvoiceNotFound)?;

        self.invoice_repo
            .mark_paid(id, razorpay_payment_id, razorpay_order_id)
            .await
    }

    pub async fn create_razorpay_order(
        &self,
        invoice_id: Uuid,
    ) -> Result<PaymentOrder, BillingError> {
        let invoice = self
            .invoice_repo
            .find_by_id(invoice_id)
            .await?
            .ok_or(BillingError::InvoiceNotFound)?;

        // Convert INR to paise for Razorpay
        let amount_paise = invoice.total_inr as i64 * 100;

        self.gateway
            .create_order(amount_paise, invoice_id)
            .await
    }
}
