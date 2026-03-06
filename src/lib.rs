#![cfg(mobile)]

use tauri::{plugin::{Builder, TauriPlugin}, Manager, Runtime, Emitter};
use tauri::async_runtime::spawn_blocking;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::plugin::PluginHandle;
use crate::models::{IntegrityTokenArgs}; 

pub use models::*;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};
use serde::Deserialize;



#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "codes.dreaming.plugin.mobile_payments";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_mobile-payments);

/// Access to the mobile-payments APIs.
pub struct MobilePayments<R: Runtime>(PluginHandle<R>);


#[derive(Deserialize)]
struct IntegrityResponse {
    token: String,
}


impl<R: Runtime> MobilePayments<R> {

    pub fn destroy(&self) -> crate::Result<()> {
        self
            .0
            .run_mobile_plugin("destroy", ())
            .map_err(Into::into)
    }

    pub async fn get_integrity_token(&self, nonce: String) -> crate::Result<String> {
        spawn_blocking({
            let app = self.0.clone();
            let args = IntegrityTokenArgs { nonce };
            move || {
                // 2. Deserialize the JSObject into our struct
                let res: IntegrityResponse = app.run_mobile_plugin("getIntegrityToken", args)?;
                
                // 3. Return just the string token
                Ok(res.token)
            }
        })
        .await?
    }

    pub async fn start_connection(&self) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            move || {
                app
                    .run_mobile_plugin("startConnection", ())
                    .map_err(Into::into)
            }
        }).await?
    }

    pub async fn purchase(&self, payload: PurchaseRequest) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            move || {
                app
                    .run_mobile_plugin("purchase", payload)
                    .map_err(Into::into)
            }
        }).await?
    }

    pub async fn update_subscription(&self, payload: UpdateSubscriptionRequest) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            // Ensure default replacement mode if None is provided from JS/TS
            let payload_with_default = UpdateSubscriptionRequest {
                replacement_mode: payload.replacement_mode.or_else(|| Some("DEFERRED".to_string())), // Default to DEFERRED if not specified
                ..payload
            };
            move || {
                app
                    .run_mobile_plugin("updateSubscription", payload_with_default) // Calls the 'updateSubscription' command in Kotlin
                    .map_err(Into::into)
            }
        }).await?
    }

    pub async fn get_active_subscription_purchase_token(&self, product_id: String) -> crate::Result<Option<String>> {
        spawn_blocking({
            let app = self.0.clone();
            let args = serde_json::json!({ "productId": product_id });
            move || {
                let res: serde_json::Value = app.run_mobile_plugin("getActiveSubscriptionPurchaseToken", args)?;
                Ok(res.get("purchaseToken").and_then(|v| v.as_str().map(|s| s.to_string())))
            }
        })
        .await?
    }

    pub async fn get_product_price(&self, payload: ProductPriceRequest) -> crate::Result<ProductDetail> {
        spawn_blocking({
            let app = self.0.clone();
            move || {
                app
                    .run_mobile_plugin("getProductPrice", payload)
                    .map_err(Into::into)
            }
        }).await?
    }


    pub async fn set_update_event_handler(&self, handler: Channel) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            move || app.run_mobile_plugin("setUpdateEventHandler", crate::models::SetEventHandlerArgs { handler })
                    .map_err(Into::into)
        }).await?
    }

    pub async fn check_for_app_update(&self, args: UpdateCheckArgs) -> crate::Result<UpdateCheck> {
        spawn_blocking({
            let app = self.0.clone();
            move || app.run_mobile_plugin("checkForAppUpdate", args)
                    .map_err(Into::into)
        }).await?
    }

    pub async fn start_app_update(&self, args: UpdateCheckArgs) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            move || app.run_mobile_plugin("startAppUpdate", args)
                    .map_err(Into::into)
        }).await?
    }

    pub async fn complete_flexible_update(&self) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            move || app.run_mobile_plugin::<()>("completeFlexibleUpdate", ())
                    .map_err(Into::into)
        }).await?
    }


    pub async fn set_fullscreen(&self, args: SetFullscreenArgs) -> crate::Result<()> {
        spawn_blocking({
            let app = self.0.clone();
            move || {
                app
                    // This matches the @Command function name in Kotlin
                    .run_mobile_plugin("setFullscreen", args) 
                    .map_err(Into::into)
            }
        }).await?
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mobile-payments APIs.
pub trait MobilePaymentsExt<R: Runtime> {
    fn mobile_payments(&self) -> &MobilePayments<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MobilePaymentsExt<R> for T {
    fn mobile_payments(&self) -> &MobilePayments<R> {
        self.state::<MobilePayments<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>(args: InitRequest) -> TauriPlugin<R> {
    Builder::new("mobile-payments")
        .invoke_handler(tauri::generate_handler![commands::start_connection, commands::purchase, commands::get_product_price, commands::update_subscription, commands::get_active_subscription_purchase_token, commands::set_update_event_handler, commands::check_for_app_update, commands::start_app_update, commands::complete_flexible_update, commands::set_fullscreen, commands::get_auth_payload])
        .setup(|app, api| {
            #[cfg(target_os = "android")]
                let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "MobilePaymentsPlugin")?;
            #[cfg(target_os = "ios")]
                let handle = api.register_ios_plugin(init_plugin_mobile - payments)?;

            handle
                .run_mobile_plugin::<()>("setEventHandler", SetEventHandlerArgs {
                    handler: Channel::new({
                        let app = app.clone();
                        move |event| {
                            println!("got channel event: {:?}", event);

                            let InvokeResponseBody::Json(json) = event else {
                                return Err(anyhow::anyhow!("invalid event").into());
                            };

                            let _ = app.emit("mobile-payments://event", json);
                            Ok(())
                        }
                    })
                })
                .expect("failed to set event handler");

            /* ---------- update events (NEW) ---------- */
            handle.run_mobile_plugin::<()>("setUpdateEventHandler", SetEventHandlerArgs {
                handler: Channel::new({
                    let app = app.clone();
                    move |event| {
                        let tauri::ipc::InvokeResponseBody::Json(json) = event else {
                            return Err(anyhow::anyhow!("invalid event").into());
                        };
                        let _ = app.emit("mobile-payments://update", json);
                        Ok(())
                    }
                })
            })?;


            handle
                .run_mobile_plugin::<()>("init", args)
                .expect("failed to initialize mobile-payments plugin");





            app.manage(MobilePayments(handle));

            Ok(())
        })
        .build()
}
