use tauri::{AppHandle, command, Runtime};
use tauri::ipc::Channel;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

use crate::{
    MobilePaymentsExt, ProductDetail, ProductPriceRequest, PurchaseRequest, 
    UpdateSubscriptionRequest, ActiveSubTokenArgs, UpdateCheckArgs, 
    UpdateCheck, Result, SetFullscreenArgs, AuthPayload
};


#[command]
pub(crate) async fn get_auth_payload<R: Runtime>(app: AppHandle<R>) -> Result<AuthPayload> {
    
    // ==========================================
    // 📱 ANDROID LOGIC
    // ==========================================
    #[cfg(target_os = "android")]
    {
        // 1. Generate UUID
        let uuid = Uuid::new_v4().to_string();

        // 2. Hash for Nonce
        let mut hasher = Sha256::new();
        hasher.update(uuid.as_bytes());
        let hash_result = hasher.finalize();
        let nonce = URL_SAFE_NO_PAD.encode(hash_result);

        // 3. Call Kotlin (via lib.rs helper function)
        let token = app.mobile_payments().get_integrity_token(nonce).await?;

        Ok(AuthPayload {
            platform: "android".to_string(),
            device_id: uuid,
            integrity_token: Some(token),
        })
    }

    // ==========================================
    // 💻 DESKTOP LOGIC
    // ==========================================
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use machine_uid;
        use keyring::Entry;

        // 1. Get Hardware ID
        let hw_id = machine_uid::get().map_err(|e| crate::Error::PluginError(e.to_string()))?; 

        // 2. Get/Set Secret in OS Keyring
        // let entry = Entry::new("qzero_ai_credits", "device_secret")
        //     .map_err(|e| crate::Error::PluginError(e.to_string()))?; // You might need to map error types

        let entry = keyring::Entry::new("q-zero-ai", "device-secret")?;


        let secret_uuid = match entry.get_password() {
            Ok(pw) => pw,
            Err(_) => {
                let new_uuid = Uuid::new_v4().to_string();
                let _ = entry.set_password(&new_uuid);
                new_uuid
            }
        };

        // 3. Combine & Hash
        let combined = format!("{}-{}", hw_id, secret_uuid);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let fingerprint = format!("{:x}", hasher.finalize());

        Ok(AuthPayload {
            platform: "desktop".to_string(),
            device_id: fingerprint,
            integrity_token: None,
        })
    }
}


#[command]
pub(crate) async fn start_connection<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.mobile_payments().start_connection().await
}

#[command]
pub(crate) async fn purchase<R: Runtime>(app: AppHandle<R>, args: PurchaseRequest) -> Result<()> {
    app.mobile_payments().purchase(args).await
}

#[command]
pub(crate) async fn get_product_price<R: Runtime>(app: AppHandle<R>, args: ProductPriceRequest) -> Result<ProductDetail> {
    app.mobile_payments().get_product_price(args).await
}

#[command]
pub(crate) async fn update_subscription<R: Runtime>(app: AppHandle<R>, args: UpdateSubscriptionRequest) -> Result<()> {
    app.mobile_payments().update_subscription(args).await
}

#[command]
pub(crate) async fn get_active_subscription_purchase_token<R: Runtime>(
    app: AppHandle<R>,
    args: ActiveSubTokenArgs,
) -> Result<Option<String>> {
    app.mobile_payments().get_active_subscription_purchase_token(args.product_id).await
}


#[command]
pub(crate) async fn set_update_event_handler<R: Runtime>(
    app: AppHandle<R>,
    handler: Channel    // passed straight from JS
) -> Result<()> {
    app.mobile_payments().set_update_event_handler(handler).await
}

#[command]
pub(crate) async fn check_for_app_update<R: Runtime>(
    app: AppHandle<R>,
    args: UpdateCheckArgs
) -> Result<UpdateCheck> {
    app.mobile_payments().check_for_app_update(args).await
}

#[command]
pub(crate) async fn start_app_update<R: Runtime>(
    app: AppHandle<R>,
    args: UpdateCheckArgs
) -> Result<()> {
    app.mobile_payments().start_app_update(args).await
}

#[command]
pub(crate) async fn complete_flexible_update<R: Runtime>(
    app: AppHandle<R>
) -> Result<()> {
    app.mobile_payments().complete_flexible_update().await
}

#[command]
pub(crate) async fn set_fullscreen<R: Runtime>(
    app: AppHandle<R>,
    hide_status_bar: bool,
    hide_navigation_bar: bool
) -> Result<()> {
    // Re-pack them into the struct to pass to the internal logic
    let args = SetFullscreenArgs { 
        hide_status_bar, 
        hide_navigation_bar 
    };
    app.mobile_payments().set_fullscreen(args).await
}