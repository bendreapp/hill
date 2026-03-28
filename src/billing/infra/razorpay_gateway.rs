use async_trait::async_trait;
use uuid::Uuid;

use crate::billing::domain::entity::PaymentOrder;
use crate::billing::domain::error::BillingError;
use crate::billing::domain::port::PaymentGatewayPort;

pub struct RazorpayGateway {
    key_id: String,
    key_secret: String,
    client: reqwest::Client,
}

impl RazorpayGateway {
    pub fn new(key_id: String, key_secret: String) -> Self {
        Self {
            key_id,
            key_secret,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(serde::Serialize)]
struct CreateOrderRequest {
    amount: i64,
    currency: String,
    receipt: String,
}

#[derive(serde::Deserialize)]
struct RazorpayOrderResponse {
    id: String,
    amount: i64,
    currency: String,
}

#[async_trait]
impl PaymentGatewayPort for RazorpayGateway {
    async fn create_order(
        &self,
        amount_paise: i64,
        invoice_id: Uuid,
    ) -> Result<PaymentOrder, BillingError> {
        let request_body = CreateOrderRequest {
            amount: amount_paise,
            currency: "INR".to_string(),
            receipt: invoice_id.to_string(),
        };

        let response = self
            .client
            .post("https://api.razorpay.com/v1/orders")
            .basic_auth(&self.key_id, Some(&self.key_secret))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| BillingError::PaymentFailed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(BillingError::PaymentFailed(format!(
                "Razorpay returned {}: {}",
                status, body
            )));
        }

        let order: RazorpayOrderResponse = response
            .json()
            .await
            .map_err(|e| BillingError::PaymentFailed(format!("Failed to parse response: {}", e)))?;

        Ok(PaymentOrder {
            razorpay_order_id: order.id,
            amount_paise: order.amount,
            currency: order.currency,
            invoice_id,
        })
    }
}
