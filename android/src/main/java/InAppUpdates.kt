package codes.dreaming.plugin.mobile_payments

import android.app.Activity
import android.content.Intent
import com.google.android.play.core.appupdate.*  // AppUpdateManager / info / options
import com.google.android.play.core.install.InstallStateUpdatedListener
import com.google.android.play.core.install.model.InstallStatus
import com.google.android.play.core.install.model.AppUpdateType
import com.google.android.play.core.install.model.UpdateAvailability
import kotlinx.coroutines.suspendCancellableCoroutine
import app.tauri.plugin.Channel
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

internal class InAppUpdates(private val activity: Activity) {

    companion object { const val REQUEST_CODE = 5173 }

    private val manager = AppUpdateManagerFactory.create(activity)
    private var listener: InstallStateUpdatedListener? = null
    private var channel: Channel? = null

    fun setChannel(ch: Channel) { channel = ch }

    /* ---------- public suspend helpers ---------- */

    suspend fun check(type: Int): AppUpdateInfo? =
        suspendCancellableCoroutine<AppUpdateInfo> { cont ->
            manager.appUpdateInfo
                .addOnSuccessListener { info -> cont.resume(info) }        // explicit λ param
                .addOnFailureListener { e -> cont.resumeWithException(e) }
        }.takeIf { info ->
            info.updateAvailability() == UpdateAvailability.UPDATE_AVAILABLE &&
            info.isUpdateTypeAllowed(type)
        }

    fun start(info: AppUpdateInfo, type: Int) {
        manager.startUpdateFlowForResult(
            info,
            activity,
            AppUpdateOptions.newBuilder(type).build(),
            REQUEST_CODE
        )
    }

    fun completeFlexible() = manager.completeUpdate()

    /* ---------- lifecycle hooks ---------- */

    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode != REQUEST_CODE) return
        channel?.sendObject(mapOf("updateActivityResult" to resultCode))
    }

    fun onResume() {
        manager.appUpdateInfo.addOnSuccessListener { info ->
            if (info.updateAvailability() ==
                    UpdateAvailability.DEVELOPER_TRIGGERED_UPDATE_IN_PROGRESS) {
                start(info, AppUpdateType.IMMEDIATE)
            }
            if (info.installStatus() == InstallStatus.DOWNLOADED) {
                channel?.sendObject(mapOf("updateReadyToInstall" to true))
            }
        }
    }

    fun registerProgressListener() {
        if (listener != null) return
        listener = InstallStateUpdatedListener { state ->
            channel?.sendObject(
                UpdateProgressMessage(
                    state.installStatus(),
                    state.bytesDownloaded(),
                    state.totalBytesToDownload()
                )
            )
        }.also(manager::registerListener)
    }

    fun unregisterProgressListener() {
        listener?.let(manager::unregisterListener)
        listener = null
    }

    /** data class we push through the Channel */
    data class UpdateProgressMessage(
        val status: Int,
        val bytesDownloaded: Long,
        val totalBytes: Long
    )
}
