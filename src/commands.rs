use tauri::{AppHandle, command, Runtime};
use crate::{MobilePaymentsExt, ProductDetail, ProductPriceRequest, PurchaseRequest, UpdateSubscriptionRequest, ActiveSubTokenArgs, UpdateCheckArgs, UpdateCheck, Channel, Result};

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