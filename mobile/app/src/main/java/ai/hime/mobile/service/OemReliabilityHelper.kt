package ai.hime.mobile.service

import android.annotation.SuppressLint
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.PowerManager
import android.provider.Settings

/**
 * OEM Reliability & Battery Optimization Guide Helper (16-MOBILE-PARITY.md §6).
 * Handles OEM-specific background kill prevention for Xiaomi, Oppo, Vivo, Realme, Samsung, Huawei.
 */
object OemReliabilityHelper {

    enum class OemBrand {
        XIAOMI,
        OPPO,
        VIVO,
        REALME,
        SAMSUNG,
        HUAWEI,
        GENERIC
    }

    fun detectOem(): OemBrand {
        val manufacturer = Build.MANUFACTURER.lowercase()
        return when {
            manufacturer.contains("xiaomi") || manufacturer.contains("redmi") || manufacturer.contains("poco") -> OemBrand.XIAOMI
            manufacturer.contains("oppo") -> OemBrand.OPPO
            manufacturer.contains("vivo") || manufacturer.contains("iqoo") -> OemBrand.VIVO
            manufacturer.contains("realme") -> OemBrand.REALME
            manufacturer.contains("samsung") -> OemBrand.SAMSUNG
            manufacturer.contains("huawei") || manufacturer.contains("honor") -> OemBrand.HUAWEI
            else -> OemBrand.GENERIC
        }
    }

    /**
     * Attempts to open OEM-specific Autostart / Background Manager settings.
     * Returns true if intent was resolved and launched, false if fallback is needed.
     */
    fun openAutostartSettings(context: Context): Boolean {
        val brand = detectOem()
        val intentList = when (brand) {
            OemBrand.XIAOMI -> listOf(
                Intent().setComponent(ComponentName("com.miui.securitycenter", "com.miui.permcenter.autostart.AutoStartManagementActivity")),
                Intent("miui.intent.action.OP_AUTO_START").addCategory(Intent.CATEGORY_DEFAULT)
            )
            OemBrand.OPPO, OemBrand.REALME -> listOf(
                Intent().setComponent(ComponentName("com.coloros.safecenter", "com.coloros.safecenter.permission.startup.StartupAppListActivity")),
                Intent().setComponent(ComponentName("com.oppo.safe", "com.oppo.safe.permission.startup.StartupAppListActivity")),
                Intent().setComponent(ComponentName("com.coloros.safecenter", "com.coloros.safecenter.startupapp.StartupAppListActivity"))
            )
            OemBrand.VIVO -> listOf(
                Intent().setComponent(ComponentName("com.iqoo.secure", "com.iqoo.secure.ui.phoneoptimize.AddWhiteListActivity")),
                Intent().setComponent(ComponentName("com.vivo.permissionmanager", "com.vivo.permissionmanager.activity.BgStartUpManagerActivity")),
                Intent().setComponent(ComponentName("com.iqoo.secure", "com.iqoo.secure.ui.phoneoptimize.BgStartUpManager"))
            )
            OemBrand.SAMSUNG -> listOf(
                Intent().setComponent(ComponentName("com.samsung.android.lool", "com.samsung.android.sm.battery.ui.BatteryActivity")),
                Intent().setComponent(ComponentName("com.samsung.android.sm", "com.samsung.android.sm.ui.battery.BatteryActivity"))
            )
            OemBrand.HUAWEI -> listOf(
                Intent().setComponent(ComponentName("com.huawei.systemmanager", "com.huawei.systemmanager.startupmgr.ui.StartupNormalAppListActivity")),
                Intent().setComponent(ComponentName("com.huawei.systemmanager", "com.huawei.systemmanager.optimize.bootstart.BootStartActivity"))
            )
            OemBrand.GENERIC -> emptyList()
        }

        for (intent in intentList) {
            try {
                intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                context.startActivity(intent)
                return true
            } catch (_: Exception) {
                // Try next variant
            }
        }

        // Fallback to app details
        return openAppDetails(context)
    }

    /**
     * Opens battery optimization exemption request dialog or system list.
     */
    @SuppressLint("BatteryLife")
    fun requestIgnoreBatteryOptimizations(context: Context) {
        val powerManager = context.getSystemService(Context.POWER_SERVICE) as? PowerManager
        val packageName = context.packageName

        if (powerManager != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            if (!powerManager.isIgnoringBatteryOptimizations(packageName)) {
                try {
                    val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                        data = Uri.parse("package:$packageName")
                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    }
                    context.startActivity(intent)
                    return
                } catch (_: Exception) {
                    // Fallback to generic optimization settings
                }
            }
        }

        try {
            val intent = Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        } catch (_: Exception) {
            openAppDetails(context)
        }
    }

    private fun openAppDetails(context: Context): Boolean {
        return try {
            val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                data = Uri.parse("package:${context.packageName}")
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
            true
        } catch (_: Exception) {
            false
        }
    }

    /**
     * Indonesian step-by-step guidance for users based on OEM.
     */
    fun getOemGuidanceText(): String {
        return when (detectOem()) {
            OemBrand.XIAOMI -> "Xiaomi/Redmi: Aktifkan 'Mulai Otomatis' di Keamanan, dan ubah Penghemat Baterai ke 'Tidak ada batasan'."
            OemBrand.OPPO -> "Oppo: Izinkan 'Mulai Otomatis' di Pengelola Ponsel, dan aktifkan 'Izinkan aktivitas latar belakang'."
            OemBrand.VIVO -> "Vivo/iQOO: Buka Pengelola Izin → 'Mulai Otomatis', dan tandai Hime di daftar aplikasi prioritas."
            OemBrand.REALME -> "Realme: Aktifkan 'Mulai Otomatis' dan izinkan berjalan di latar belakang tanpa optimasi."
            OemBrand.SAMSUNG -> "Samsung: Masuk ke Perawatan Perangkat → Baterai → Batas Penggunaan Latar Belakang → tambahkan Hime ke 'Aplikasi yang tidak pernah tidur'."
            OemBrand.HUAWEI -> "Huawei: Masuk ke Pengelola Peluncuran Aplikasi → ubah Hime ke 'Kelola Secara Manual' dan aktifkan semua izin."
            OemBrand.GENERIC -> "Atur Penghemat Baterai untuk Hime menjadi 'Tanpa Batasan' agar alarm dan penegakan fokus berjalan tepat waktu."
        }
    }
}
