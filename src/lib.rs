use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime, Emitter,
    ipc::{Channel, InvokeResponseBody},
    plugin::PluginHandle
};
use tauri::async_runtime::spawn_blocking;
use serde::Deserialize;

use crate::models::{IntegrityTokenArgs};

// Export models and errors so commands can use them
pub use models::*;
pub use error::{Error, Result};

mod commands;
mod error;
mod models;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "codes.dreaming.plugin.mobile_payments";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_mobile_payments);

// ==========================================
// 1. UNIVERSAL STRUCT
// ==========================================

pub struct MobilePayments<R: Runtime> {
    // We only hold the plugin handle on mobile.
    // On desktop, we use PhantomData to keep the compiler happy.
    #[cfg(mobile)]
    handle: PluginHandle<R>,
    
    #[cfg(desktop)]
    _marker: std::marker::PhantomData<R>,
}

#[derive(Deserialize)]
struct IntegrityResponse {
    token: String,
}

// ==========================================
// 2. IMPLEMENTATION
// ==========================================

impl<R: Runtime> MobilePayments<R> {
    
    pub fn destroy(&self) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            self.handle.run_mobile_plugin("destroy", ()).map_err(Into::into)
        }
        #[cfg(desktop)]
        Ok(())
    }

    // Helper to get raw string token from mobile plugin
    pub async fn get_integrity_token(&self, nonce: String) -> crate::Result<String> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                let args = IntegrityTokenArgs { nonce };
                move || {
                    let res: IntegrityResponse = app.run_mobile_plugin("getIntegrityToken", args)?;
                    Ok(res.token)
                }
            })
            .await?
        }
        #[cfg(desktop)]
        {
            // This path shouldn't be reached if commands.rs handles desktop logic,
            // but for safety, return an error.
            Err(Error::PluginError("Native Integrity Token only available on Mobile".into()))
        }
    }

    pub async fn start_connection(&self) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("startConnection", ()).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        Ok(())
    }

    pub async fn purchase(&self, payload: PurchaseRequest) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("purchase", payload).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        {
            // ⚠️ Safe Fallback: Tell frontend it's not supported instead of crashing
            Err(Error::PluginError("In-App Purchases are not supported on Desktop".into()))
        }
    }

    pub async fn update_subscription(&self, payload: UpdateSubscriptionRequest) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                // Default to DEFERRED if missing
                let payload_with_default = UpdateSubscriptionRequest {
                    replacement_mode: payload.replacement_mode.or_else(|| Some("DEFERRED".to_string())),
                    ..payload
                };
                move || app.run_mobile_plugin("updateSubscription", payload_with_default).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        {
            Err(Error::PluginError("Subscriptions not supported on Desktop".into()))
        }
    }

    pub async fn get_active_subscription_purchase_token(&self, product_id: String) -> crate::Result<Option<String>> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                let args = serde_json::json!({ "productId": product_id });
                move || {
                    let res: serde_json::Value = app.run_mobile_plugin("getActiveSubscriptionPurchaseToken", args)?;
                    Ok(res.get("purchaseToken").and_then(|v| v.as_str().map(|s| s.to_string())))
                }
            }).await?
        }
        #[cfg(desktop)]
        Ok(None)
    }

    pub async fn get_product_price(&self, payload: ProductPriceRequest) -> crate::Result<ProductDetail> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("getProductPrice", payload).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        {
            Err(Error::PluginError("Product prices not supported on Desktop".into()))
        }
    }

    pub async fn set_update_event_handler(&self, handler: Channel) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("setUpdateEventHandler", crate::models::SetEventHandlerArgs { handler })
                    .map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        Ok(())
    }

    pub async fn check_for_app_update(&self, args: UpdateCheckArgs) -> crate::Result<UpdateCheck> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("checkForAppUpdate", args).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        {
            // ⚠️ Safe Fallback: Pretend no update is available
            Ok(UpdateCheck {
                update_available: false,
                available_version_code: None,
                staleness_days: None,
                priority: None,
                is_immediate_allowed: Some(false),
                is_flexible_allowed: Some(false),
            })
        }
    }

    pub async fn start_app_update(&self, args: UpdateCheckArgs) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("startAppUpdate", args).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        {
            Err(Error::PluginError("In-App Updates not supported on Desktop".into()))
        }
    }

    pub async fn complete_flexible_update(&self) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin::<()>("completeFlexibleUpdate", ()).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        Ok(())
    }

    pub async fn set_fullscreen(&self, args: SetFullscreenArgs) -> crate::Result<()> {
        #[cfg(mobile)]
        {
            spawn_blocking({
                let app = self.handle.clone();
                move || app.run_mobile_plugin("setFullscreen", args).map_err(Into::into)
            }).await?
        }
        #[cfg(desktop)]
        Ok(())
    }
}

// ==========================================
// 3. EXTENSION TRAIT
// ==========================================

pub trait MobilePaymentsExt<R: Runtime> {
    fn mobile_payments(&self) -> &MobilePayments<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MobilePaymentsExt<R> for T {
    fn mobile_payments(&self) -> &MobilePayments<R> {
        self.state::<MobilePayments<R>>().inner()
    }
}

// ==========================================
// 4. INITIALIZATION
// ==========================================

pub fn init<R: Runtime>(args: InitRequest) -> TauriPlugin<R> {
    Builder::new("mobile-payments")
        .invoke_handler(tauri::generate_handler![
            commands::start_connection, 
            commands::purchase, 
            commands::get_product_price, 
            commands::update_subscription, 
            commands::get_active_subscription_purchase_token, 
            commands::set_update_event_handler, 
            commands::check_for_app_update, 
            commands::start_app_update, 
            commands::complete_flexible_update, 
            commands::set_fullscreen, 
            commands::get_auth_payload
        ])
        .setup(|app, api| {
            
            // --- MOBILE SETUP ---
            #[cfg(mobile)]
            {
                #[cfg(target_os = "android")]
                let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "MobilePaymentsPlugin")?;
                #[cfg(target_os = "ios")]
                let handle = api.register_ios_plugin(init_plugin_mobile_payments)?;

                // Event Handlers
                handle.run_mobile_plugin::<()>("setEventHandler", SetEventHandlerArgs {
                    handler: Channel::new({
                        let app = app.clone();
                        move |event| {
                            if let InvokeResponseBody::Json(json) = event {
                                let _ = app.emit("mobile-payments://event", json);
                            }
                            Ok(())
                        }
                    })
                })?;

                handle.run_mobile_plugin::<()>("setUpdateEventHandler", SetEventHandlerArgs {
                    handler: Channel::new({
                        let app = app.clone();
                        move |event| {
                            if let InvokeResponseBody::Json(json) = event {
                                let _ = app.emit("mobile-payments://update", json);
                            }
                            Ok(())
                        }
                    })
                })?;

                handle.run_mobile_plugin::<()>("init", args)?;

                app.manage(MobilePayments { handle });
            }

            // --- DESKTOP SETUP ---
            #[cfg(desktop)]
            {
                // Just register the state so commands don't crash when calling app.mobile_payments()
                app.manage(MobilePayments { 
                    _marker: std::marker::PhantomData 
                });
            }

            Ok(())
        })
        .build()
}