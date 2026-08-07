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

private var isConnecting = false 

class MobilePayments(private val activity: Activity) {
    private var billingClient: BillingClient? = null
    private var channel: Channel? = null

    // Safe helper to extract subResponseCode without compiler issues
    private fun extractSubResponseCode(billingResult: BillingResult): Int {
        return try {
            val method = billingResult.javaClass.getMethod("getSubResponseCode")
            (method.invoke(billingResult) as? Int) ?: 0
        } catch (e: Exception) {
            0
        }
    }

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
                            "subResponseCode" to extractSubResponseCode(billingResult),
                            "debugMessage" to billingResult.debugMessage
                        ),
                        "purchases" to purchases.orEmpty().map { p ->
                            mapOf(
                                "orderId" to p.orderId,
                                "packageName" to p.packageName,
                                "products" to p.products,
                                "purchaseToken" to p.purchaseToken,
                                "purchaseTime" to p.purchaseTime,
                                "acknowledged" to p.isAcknowledged,
                                "originalJson" to p.originalJson
                            )
                        }
                    )
                )
            }
            enablePendingPurchases(
                PendingPurchasesParams.newBuilder()
                    .enableOneTimeProducts()
                    .build()
            )
        }.build()
    }

    fun setEventHandler(channel: Channel) {
        this.channel = channel
    }

    suspend fun startConnection() {
        if (billingClient == null) {
            println("MobilePayments: Auto-initializing BillingClient...")
            init(false) 
        }

        if (billingClient?.isReady == true) return
        if (isConnecting) return

        val client = billingClient ?: throw IllegalStateException("BillingClient not initialized.")

        isConnecting = true

        try {
            suspendCancellableCoroutine<Unit> { continuation ->
                client.startConnection(object : BillingClientStateListener {
                    override fun onBillingSetupFinished(billingResult: BillingResult) {
                        isConnecting = false
                        if (billingResult.responseCode == BillingResponseCode.OK) {
                            println("MobilePayments: Billing connected successfully!")
                            continuation.resume(Unit)
                        } else {
                            println("MobilePayments: Billing setup failed: ${billingResult.responseCode}")
                            continuation.cancel(
                                CancellationException(
                                    "Billing setup failed: ${billingResult.responseCode}"
                                )
                            )
                        }
                    }

                    override fun onBillingServiceDisconnected() {
                        isConnecting = false
                        println("MobilePayments: Billing service disconnected.")
                        channel?.sendObject(
                            mapOf(
                                "isDisconnectError" to true,
                                "message" to "Billing service disconnected."
                            )
                        )
                    }
                })
            }
        } catch (e: Exception) {
            isConnecting = false 
            throw e
        }
    }

    fun endConnection() {
        val client = billingClient ?: return 

        try {
            client.endConnection()
        } catch (e: Exception) {
            println("Error during BillingClient cleanup: ${e.message}")
        } finally {
            billingClient = null
            channel = null
        }
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

    fun extractPriceInfo(
        productDetails: ProductDetails, 
        offerId: String? = null,
        basePlanId: String? = null
    ): PriceInfo {
        productDetails.subscriptionOfferDetails?.let { offers ->
            // Match both basePlanId and offerId
            val targetOffer = offers.firstOrNull { offer ->
                val matchesBasePlan = basePlanId == null || offer.basePlanId == basePlanId
                val matchesOffer = if (offerId != null) {
                    offer.offerId == offerId
                } else {
                    offer.offerId == null
                }
                matchesBasePlan && matchesOffer
            } ?: offers.firstOrNull { offer ->
                basePlanId == null || offer.basePlanId == basePlanId
            } ?: offers.firstOrNull()

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

        productDetails.oneTimePurchaseOfferDetails?.let { offer ->
             return PriceInfo(offer.formattedPrice, null, offer.priceCurrencyCode, offer.priceAmountMicros)
        }

        throw IllegalStateException("Failed to extract price info for ${productDetails.productId}")
    }

    suspend fun launchPurchaseFlow(
        productId: String,
        productType: String,
        obfuscatedAccountId: String?,
        updateParams: BillingFlowParams.SubscriptionUpdateParams?,
        offerId: String? = null,
        basePlanId: String? = null
    ): BillingResult {
        val client = billingClient ?: throw IllegalStateException("BillingClient not initialized.")

        val productDetails = getProductDetails(productId, productType) 

        val productDetailsParamsBuilder = BillingFlowParams.ProductDetailsParams.newBuilder()
            .setProductDetails(productDetails)

        if (productType == ProductType.SUBS) {
            val offers = productDetails.subscriptionOfferDetails
            if (!offers.isNullOrEmpty()) {
                // Match both basePlanId and offerId
                val targetOffer = offers.firstOrNull { offer ->
                    val matchesBasePlan = basePlanId == null || offer.basePlanId == basePlanId
                    val matchesOffer = if (offerId != null) {
                        offer.offerId == offerId
                    } else {
                        offer.offerId == null
                    }
                    matchesBasePlan && matchesOffer
                } ?: offers.firstOrNull { offer ->
                    basePlanId == null || offer.basePlanId == basePlanId
                } ?: offers.first()

                productDetailsParamsBuilder.setOfferToken(targetOffer.offerToken)
            } else {
                System.err.println("Warning: No offers found for subscription product $productId")
            }
        }

        val billingFlowParamsBuilder = BillingFlowParams.newBuilder()
            .setProductDetailsParamsList(listOf(productDetailsParamsBuilder.build()))
            .apply {
                obfuscatedAccountId?.let { setObfuscatedAccountId(it) }
                updateParams?.let { setSubscriptionUpdateParams(it) }
            }

        val billingFlowParams = billingFlowParamsBuilder.build()

        return client.launchBillingFlow(activity, billingFlowParams)
    }
}