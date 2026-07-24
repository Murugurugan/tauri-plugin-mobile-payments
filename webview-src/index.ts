import {invoke, Channel} from "@tauri-apps/api/core";
import {PaymentEvent, ProductDetail, PurchaseRequest, ProductPriceRequest, UpdateSubscriptionRequest, AuthPayload, ActiveSubTokenArgs } from "./bindings";
import type { UpdateType as UpdateTypeT, UpdateProgress, UpdateCheck } from "./bindings";

import {EventCallback, listen, UnlistenFn} from "@tauri-apps/api/event";

export const enum BillingSubResponseCode {
    NO_APPLICABLE_SUB_RESPONSE_CODE = 0,
    PAYMENT_DECLINED_DUE_TO_INSUFFICIENT_FUNDS = 1,
    USER_INELIGIBLE = 2,
}

export const enum SubscriptionReplacementMode {
    /** The replacement takes effect immediately, and the remaining time will be prorated and credited to the user. */
    IMMEDIATE_WITH_TIME_PRORATION = "IMMEDIATE_WITH_TIME_PRORATION",
    /** The replacement takes effect immediately, and the billing cycle remains the same. The price for the remaining period will be charged. */
    IMMEDIATE_AND_CHARGE_PRORATED_PRICE = "IMMEDIATE_AND_CHARGE_PRORATED_PRICE",
    /** The replacement takes effect immediately, and the new price will be charged on next recurrence time. The remaining time is lost. */
    IMMEDIATE_WITHOUT_PRORATION = "IMMEDIATE_WITHOUT_PRORATION",
    /** The replacement takes effect immediately, and the user is charged full price of new plan. The remaining time is lost. */
    IMMEDIATE_AND_CHARGE_FULL_PRICE = "IMMEDIATE_AND_CHARGE_FULL_PRICE",
    /** The replacement takes effect when the old plan expires. */
    DEFERRED = "DEFERRED",
}



export async function getSecureIdentity(): Promise<AuthPayload> {
    return await invoke<AuthPayload>('plugin:mobile-payments|get_auth_payload');
}

export async function startConnection() {
    try {
        await invoke('plugin:mobile-payments|start_connection');
    } catch (e) {
        // console.warn("startConnection: Not supported on this platform.");
        console.error("CRITICAL startConnection ERROR:", e); 
        throw e; 
    }
}

export async function endConnection() {
    try {
        await invoke('plugin:mobile-payments|end_connection');
    } catch (e) {
        console.warn("endConnection: Not supported on this platform.");
    }
}

export async function purchase(args: PurchaseRequest) {
    await invoke('plugin:mobile-payments|purchase', {args})
}

export async function getProductPrice(args: ProductPriceRequest) {
    return await invoke<ProductDetail>('plugin:mobile-payments|get_product_price', {args})
}

export function listenForPurchases(handler: EventCallback<any>): Promise<UnlistenFn> {
    return listen("mobile-payments://event", handler);
}

export function listenForUpdateEvents(handler: EventCallback<UpdateProgress | Record<string, unknown>>): Promise<UnlistenFn> {
    return listen("mobile-payments://update", handler);
}

export async function updateSubscription(args: {
    newProductId: string;
    oldPurchaseToken: string;
    replacementMode?: SubscriptionReplacementMode; // Use the enum/constants
    obfuscatedAccountId?: string;
}): Promise<void> {
    // Map the enum/constant to the string expected by Rust/Kotlin
    const requestArgs: UpdateSubscriptionRequest = {
        newProductId: args.newProductId,
        oldPurchaseToken: args.oldPurchaseToken,
        // Provide a default if desired, or let Rust handle it
        replacementMode: args.replacementMode ?? SubscriptionReplacementMode.DEFERRED,
        obfuscatedAccountId: args.obfuscatedAccountId,
    };
    await invoke("plugin:mobile-payments|update_subscription", { args: requestArgs });
}

export async function getActiveSubscriptionPurchaseToken(productId: string): Promise<string | null> {
    try {
        const result = await invoke<{ purchaseToken: string | null }>(
            'plugin:mobile-payments|get_active_subscription_purchase_token',
            { args: { productId } }
        );
        return result.purchaseToken ?? null;
    } catch (e) {
        return null;
    }
}


/** Runtime constants that mirror the union type. */
// export const UpdateTypes = {
//     IMMEDIATE: "IMMEDIATE",
//     FLEXIBLE:  "FLEXIBLE",
// } as const satisfies Record<UpdateType, UpdateType>;

// export const UpdateType = {
//     IMMEDIATE: "IMMEDIATE",
//     FLEXIBLE:  "FLEXIBLE",
// } as const;

/** Ask Play Core whether an update is available & allowed */
export function checkForAppUpdate(
    updateType: UpdateType = UpdateType.IMMEDIATE   // ✅ enum member
): Promise<UpdateCheck> {
    return invoke("plugin:mobile-payments|check_for_app_update", {
        args: { updateType },
    });
}

export function startAppUpdate(
    updateType: UpdateType = UpdateType.IMMEDIATE   // ✅ enum member
): Promise<void> {
    return invoke("plugin:mobile-payments|start_app_update", {
        args: { updateType },
    });
}

/** Call this after a FLEXIBLE download finishes to restart / install */
export async function completeFlexibleUpdate(): Promise<void> {
    await invoke("plugin:mobile-payments|complete_flexible_update");
}




// export function createChannel<T>(
//     callback: (payload: T) => void
// ): Promise<Channel<T>>

let updateChannel: Channel | null = null;

/**
 * Register a callback that receives every progress / status message
 * sent from the Kotlin `updates.setChannel().sendObject(...)`.
 *
 * The function returns an `unlisten()` you should call on shutdown.
 */
export async function registerUpdateEventHandler(
    callback: (data: UpdateProgress | Record<string, unknown>) => void
): Promise<() => void> {

    if (updateChannel) {
        updateChannel.onmessage = callback;
        return () => {}; // Disposer
    }

    // 1️⃣  Create a JS-side channel bound to your callback
    updateChannel = new Channel(callback);

    // 2️⃣  Give the channel object itself to Kotlin/Rust
    await invoke("plugin:mobile-payments|set_update_event_handler", {
        handler: updateChannel             // <-- just pass the Channel
    });

    // 3️⃣  Return a simple disposer
    return () => {
        if (updateChannel) {
            updateChannel.onmessage = () => {}; 
            // In a real plugin, you might also want an "unset_handler" command
        }
    };
}

export const UpdateType = {
    IMMEDIATE: "IMMEDIATE",
    FLEXIBLE:  "FLEXIBLE",
} as const satisfies Record<UpdateTypeT, UpdateTypeT>;



export type UpdateType = UpdateTypeT;


export {PurchaseRequest}

export type { UpdateProgress, UpdateCheck } from "./bindings";

// export { UpdateType } from "./bindings";