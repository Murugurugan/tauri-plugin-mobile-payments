package codes.dreaming.plugin.mobile_payments

import android.app.Activity
import app.tauri.plugin.Channel
import com.android.billingclient.api.*
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingClient.ProductType
import com.android.billingclient.api.QueryPurchasesParams
import com.android.billingclient.api.Purchase
import kotlinx.coroutines.suspendCancellableCoroutine
import java.util.concurrent.CancellationException
import kotlin.coroutines.resume

@Suppress("unused")
data class PurchasesUpdatedChannelMessage(val billingResult: BillingResult, val purchases: List<Purchase>)

data class PriceInfo(
    val formattedPrice: String?,
    val formattedFullPrice: String?,
    val currencyCode: String?,
    val priceAmountMicros: Long?
)

private data class PurchaseWire(
    val orderId:        String?,
    val packageName:    String?,
    val products:       List<String>,
    val purchaseToken:  String,
    val purchaseTime:   Long,
    val acknowledged:   Boolean,
    val originalJson:   String
)

private fun Purchase.toWire() = PurchaseWire(
    orderId,
    packageName,
    products,
    purchaseToken,
    purchaseTime,
    isAcknowledged,
    originalJson
)


class MobilePayments(private val activity: Activity) {
    private var billingClient: BillingClient? = null
    private var channel: Channel? = null

    fun init(enableAlternativeBillingOnly: Boolean) {
        billingClient?.let {
            throw IllegalStateException("BillingClient already initialized")
        }

        billingClient = BillingClient.newBuilder(activity).apply {
            setListener { billingResult, purchases ->
                channel?.sendObject(
                    mapOf(
                        "billingResult" to mapOf(
                            "responseCode" to billingResult.responseCode,
                            "debugMessage" to billingResult.debugMessage
                        ),
                        "purchases" to purchases.orEmpty().map { it.toWire() }
                    )
                )
            }
            enablePendingPurchases(PendingPurchasesParams.newBuilder().enableOneTimeProducts().build())
            if (enableAlternativeBillingOnly) {
                enableAlternativeBillingOnly()
            }
        }.build()
    }

    fun setEventHandler(channel: Channel) {
        this.channel = channel
    }

    suspend fun startConnection() {
        billingClient?.let { client ->
            suspendCancellableCoroutine<Unit> { continuation ->
                client.startConnection(object : BillingClientStateListener {
                    override fun onBillingSetupFinished(billingResult: BillingResult) {
                        if (billingResult.responseCode == BillingResponseCode.OK) {
                            continuation.resume(Unit)
                        } else {
                            continuation.cancel(CancellationException("Billing setup failed with response code: ${billingResult.responseCode}"))
                        }
                    }

                    override fun onBillingServiceDisconnected() {
                        // TODO: Implement retry logic or notify the rust side that we are disconnected.
                    }
                })
            }
        } ?: throw IllegalStateException("BillingClient not initialized.")
    }


    suspend fun getActiveSubscriptionPurchaseToken(productId: String): String? {
        val client = billingClient ?: throw IllegalStateException("BillingClient not initialized.")

        return suspendCancellableCoroutine { continuation ->
            client.queryPurchasesAsync(
                QueryPurchasesParams.newBuilder()
                    .setProductType(BillingClient.ProductType.SUBS)
                    .build()
            ) { billingResult, purchasesList ->
                if (billingResult.responseCode != BillingClient.BillingResponseCode.OK) {
                    continuation.resumeWith(Result.failure(IllegalStateException("Failed to query purchases: ${billingResult.debugMessage}")))
                    return@queryPurchasesAsync
                }
                // Find the purchase for the given productId
                val token = purchasesList.firstOrNull { it.products.contains(productId) }?.purchaseToken
                continuation.resume(token)
            }
        }
    }



    suspend fun getProductDetails(productId: String, productType: String): ProductDetails {
        val client = billingClient ?: throw IllegalStateException("BillingClient not initialized.")

        val productList = listOf(
            QueryProductDetailsParams.Product.newBuilder()
                .setProductId(productId)
                .setProductType(productType)
                .build()
        )
        val params = QueryProductDetailsParams.newBuilder().setProductList(productList).build()

        val productDetailsResult = client.queryProductDetails(params)

        if (productDetailsResult.billingResult.responseCode != BillingResponseCode.OK) {
            throw IllegalStateException("Failed to query product details: ${productDetailsResult.billingResult.debugMessage} (Code: ${productDetailsResult.billingResult.responseCode})")
        }

        return productDetailsResult.productDetailsList?.firstOrNull { it.productId == productId }
            ?: throw IllegalStateException("Product details not found for ID: $productId")
    }

    fun extractPriceInfo(productDetails: ProductDetails, offerId: String? = null): PriceInfo {
        // For Subscriptions
        productDetails.subscriptionOfferDetails?.let { offers ->
            
            val targetOffer = if (offerId != null) {
                // FIX: Throw an error if the specific offer requested is not found
                offers.firstOrNull { it.offerId == offerId }
                    ?: throw IllegalStateException("Offer ID '$offerId' not found or ineligible for product ${productDetails.productId}")
            } else {
                offers.firstOrNull { it.offerId == null } ?: offers.firstOrNull()
            }

            targetOffer?.let { offer ->
                val phases = offer.pricingPhases.pricingPhaseList
                
                val firstPhase = phases.firstOrNull()
                val lastPhase = phases.lastOrNull()

                if (firstPhase != null) {
                    var fullPrice: String? = null

                    if (lastPhase != null && lastPhase.priceAmountMicros > firstPhase.priceAmountMicros) {
                        fullPrice = lastPhase.formattedPrice
                    }

                    return PriceInfo(
                        firstPhase.formattedPrice, 
                        fullPrice, 
                        firstPhase.priceCurrencyCode, 
                        firstPhase.priceAmountMicros
                    )
                }
            }
        }

        // For One-Time Purchases
        productDetails.oneTimePurchaseOfferDetails?.let { offer ->
             return PriceInfo(offer.formattedPrice, null, offer.priceCurrencyCode, offer.priceAmountMicros)
        }

        // FIX: If it gets here and was looking for a sub, it failed.
        throw IllegalStateException("Failed to extract price info for ${productDetails.productId}")
    }

    // --- Consolidate purchase flow logic ---
    suspend fun launchPurchaseFlow(
        productId: String,
        productType: String,
        obfuscatedAccountId: String?,
        updateParams: BillingFlowParams.SubscriptionUpdateParams?,
        offerId: String? = null
    ): BillingResult {
        val client = billingClient ?: throw IllegalStateException("BillingClient not initialized.")

        // 1. Get ProductDetails for the product being purchased/updated TO
        val productDetails = getProductDetails(productId, productType) // Use the refactored method

        // 2. Build ProductDetailsParams (including offer token for SUBS)
        val productDetailsParamsBuilder = BillingFlowParams.ProductDetailsParams.newBuilder()
            .setProductDetails(productDetails)

        if (productType == ProductType.SUBS) {
            val offers = productDetails.subscriptionOfferDetails
            if (!offers.isNullOrEmpty()) {
                val targetOffer = if (offerId != null) {
                    offers.firstOrNull { it.offerId == offerId }
                        ?: throw IllegalStateException("Offer ID '$offerId' not found for product $productId")
                } else {
                    // If no offerId is passed, try to find the Base Plan (which has a null offerId)
                    // This prevents accidentally giving a free trial if you just wanted to charge full price.
                    offers.firstOrNull { it.offerId == null } ?: offers.first()
                }

                productDetailsParamsBuilder.setOfferToken(targetOffer.offerToken)
            } else {
                System.err.println("Warning: No offers found for subscription product $productId")
            }
        }

        // 3. Build BillingFlowParams
        val billingFlowParamsBuilder = BillingFlowParams.newBuilder()
            .setProductDetailsParamsList(listOf(productDetailsParamsBuilder.build()))
            .apply {
                obfuscatedAccountId?.let { setObfuscatedAccountId(it) }
                // Add update parameters ONLY if this is an upgrade/downgrade
                updateParams?.let { setSubscriptionUpdateParams(it) }
            }

        val billingFlowParams = billingFlowParamsBuilder.build()

        // 4. Launch Billing Flow
        return client.launchBillingFlow(activity, billingFlowParams)
    }

    suspend fun purchase(productId: String, productType: String, obfuscatedAccountId: String?): BillingResult {
        billingClient?.let { client ->
            val productList = listOf(
                QueryProductDetailsParams.Product.newBuilder().setProductId(productId).setProductType(productType)
                    .build()
            )

            val params = QueryProductDetailsParams.newBuilder().setProductList(productList).build()

            val productsDetails = client.queryProductDetails(params)

            if (productsDetails.billingResult.responseCode != BillingResponseCode.OK) {
                throw IllegalStateException("Billing response code: ${productsDetails.billingResult.responseCode}")
            }

            val productDetailsList =
                productsDetails.productDetailsList ?: throw IllegalStateException("Product details list is empty.")

            val productDetailsParamsList = productDetailsList.map { productDetails ->
                BillingFlowParams.ProductDetailsParams.newBuilder().setProductDetails(productDetails).apply {
                    if (productType == ProductType.SUBS) {
                        productDetails.subscriptionOfferDetails?.firstOrNull()?.offerToken?.let { setOfferToken(it) }
                    }
                }.build()
            }

            val billingFlowParams =
                BillingFlowParams.newBuilder().setProductDetailsParamsList(productDetailsParamsList).apply {
                    obfuscatedAccountId?.let { setObfuscatedAccountId(it) }
                }.build()

            return client.launchBillingFlow(activity, billingFlowParams)
        } ?: throw IllegalStateException("BillingClient not initialized.")
    }
}
