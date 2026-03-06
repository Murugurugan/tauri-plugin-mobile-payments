package codes.dreaming.plugin.mobile_payments

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import com.google.android.play.core.integrity.IntegrityManagerFactory
import com.google.android.play.core.integrity.IntegrityTokenRequest
import com.google.android.play.core.integrity.model.IntegrityErrorCode
import com.google.android.play.core.integrity.IntegrityServiceException

@InvokeArg
class TokenArgs {
    var nonce: String = ""
}

@TauriPlugin
class IdentityPlugin(private val activity: Activity) : Plugin(activity) {

    @Command
    fun getIntegrityToken(invoke: Invoke) {
        val args = invoke.parseArgs(TokenArgs::class.java)
        val nonce = args.nonce

        // Create the manager (API 1.6.0 standard)
        val integrityManager = IntegrityManagerFactory.create(activity.applicationContext)

        // Do NOT set webViewRequestMode here, as per changelog warning.
        val request = IntegrityTokenRequest.builder()
            .setNonce(nonce)
            .build()

        integrityManager.requestIntegrityToken(request)
            .addOnSuccessListener { response ->
                val ret = JSObject()
                ret.put("token", response.token())
                invoke.resolve(ret)
            }
            .addOnFailureListener { e ->
                if (e is IntegrityServiceException) {
                    when (e.errorCode) {
                        IntegrityErrorCode.PLAY_STORE_NOT_FOUND -> 
                            invoke.reject("Play Store not found.")
                        IntegrityErrorCode.NETWORK_ERROR -> 
                            invoke.reject("Network error.")
                        IntegrityErrorCode.CLIENT_TRANSIENT_ERROR -> 
                            invoke.reject("Transient error. Please retry.")
                        else -> 
                            invoke.reject("Integrity Error Code: ${e.errorCode}")
                    }
                } else {
                    invoke.reject("Integrity Error: ${e.message}")
                }
            }
    }
}