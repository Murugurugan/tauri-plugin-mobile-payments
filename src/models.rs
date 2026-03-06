use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tsync::tsync;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct IntegrityTokenArgs {
  pub nonce: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct AuthPayload {
  pub platform: String,
  pub device_id: String,
  pub integrity_token: Option<String>,
}

#[derive(Serialize)]
pub(super) struct SetEventHandlerArgs {
  pub handler: Channel
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct PurchaseRequest {
  pub product_id: String,
  pub is_sub: bool,
  pub obfuscated_account_id: Option<String>,
  pub offer_id: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct UpdateSubscriptionRequest {
    pub new_product_id: String, // ID of the target subscription tier
    pub old_purchase_token: String, // Token of the current subscription
    // #[tsync(optional)] // Make optional in TS if default is handled in Kotlin/JS
    pub replacement_mode: Option<String>, // e.g., "IMMEDIATE_WITH_TIME_PRORATION", "DEFERRED"
    pub obfuscated_account_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct ProductPriceRequest {
  pub product_id: String,
  pub sub: bool,
  pub offer_id: Option<String>
}

#[derive(Serialize)]
pub struct InitRequest {
  pub alternative_billing_only: bool
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct ActiveSubTokenArgs {
    pub product_id: String,
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
struct AccountIdentifiers {
  pub obfuscated_account_id: String,
  pub obfuscated_profile_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct ProductDetail {
  pub formatted_price: Option<String>,
  pub formatted_full_price: Option<String>,
  pub currency_code: Option<String>,
  // #[tsync(optional)] // Mark as optional in TS
  pub price_amount_micros: Option<i64>, // Use i64 for micros
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
struct Purchase {
  pub account_identifiers: AccountIdentifiers,
  pub acknowledged: bool,
  pub auto_renewing: bool,
  pub developer_payload: String,
  pub order_id: String,
  pub original_json: String,
  pub package_name: String,
  pub products: Vec<String>,
  pub purchase_state: i64,
  pub purchase_time: i64,
  pub purchase_token: String,
  pub quantity: i64,
  pub signature: String,
  pub skus: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
struct BillingResult {
  pub debug_message: String,
  pub response_code: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
struct PaymentEvent {
  pub billing_result: BillingResult,
  pub purchases: Vec<Purchase>,
}




/// Matches Kotlin’s `UpdateType` argument.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[tsync]
pub enum UpdateType {
  IMMEDIATE,
  FLEXIBLE,
}


/// Payload sent by Kotlin on every progress tick.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct UpdateProgress {
  pub status:          i32,   // com.google.android.play.core.install.InstallStatus
  pub bytes_downloaded: i64,
  pub total_bytes:      i64,
}

/// Args object for `checkForAppUpdate` and `startAppUpdate`.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct UpdateCheckArgs {
  // #[serde(default)]
  pub update_type: UpdateType,   // default = IMMEDIATE in JS side
}

/// Response from `checkForAppUpdate`.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct UpdateCheck {
  pub update_available:   bool,
  pub available_version_code: Option<i32>,
  pub staleness_days:     Option<i32>,
  pub priority:           Option<i32>,
  pub is_immediate_allowed: Option<bool>,
  pub is_flexible_allowed:  Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsync]
pub struct SetFullscreenArgs {
  pub hide_status_bar: bool,
  pub hide_navigation_bar: bool,
}