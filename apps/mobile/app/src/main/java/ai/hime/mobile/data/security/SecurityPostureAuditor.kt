package ai.hime.mobile.data.security

import android.app.KeyguardManager
import android.app.admin.DevicePolicyManager
import android.content.Context
import android.os.Build
import android.provider.Settings
import ai.hime.mobile.data.model.SecurityPosture

class SecurityPostureAuditor(private val context: Context) {

    fun auditPosture(): SecurityPosture {
        val keyguardManager = context.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
        val isDeviceSecure = keyguardManager?.isDeviceSecure ?: false

        val dpm = context.getSystemService(Context.POLICY_SERVICE) as? DevicePolicyManager
        val storageEncrypted = if (dpm != null) {
            val status = dpm.storageEncryptionStatus
            status == DevicePolicyManager.ENCRYPTION_STATUS_ACTIVE ||
                    status == DevicePolicyManager.ENCRYPTION_STATUS_ACTIVE_DEFAULT_KEY ||
                    status == DevicePolicyManager.ENCRYPTION_STATUS_ACTIVE_PER_USER
        } else {
            Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q
        }

        val unknownSourcesEnabled = try {
            val setting = Settings.Secure.getInt(
                context.contentResolver,
                Settings.Secure.INSTALL_NON_MARKET_APPS,
                0
            )
            setting == 1
        } catch (_: Exception) {
            false
        }

        val patchLevel = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            Build.VERSION.SECURITY_PATCH
        } else {
            "Unknown"
        }

        return SecurityPosture(
            isDeviceSecure = isDeviceSecure,
            securityPatchLevel = patchLevel,
            storageEncrypted = storageEncrypted,
            unknownSourcesEnabled = unknownSourcesEnabled,
            osVersion = "Android ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})"
        )
    }
}
