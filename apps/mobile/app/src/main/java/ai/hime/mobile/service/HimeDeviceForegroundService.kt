package ai.hime.mobile.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.BatteryManager
import android.os.Build
import android.os.IBinder
import android.os.StatFs
import androidx.core.app.NotificationCompat
import ai.hime.mobile.R
import ai.hime.mobile.data.model.MobileDeviceStatus
import ai.hime.mobile.data.net.MtlsClient
import ai.hime.mobile.data.security.KeystoreManager
import ai.hime.mobile.data.security.SecurityPostureAuditor
import ai.hime.mobile.ui.MainActivity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.jsonObject
import java.io.File

class HimeDeviceForegroundService : Service() {

    companion object {
        const val CHANNEL_ID = "hime_device_sync_channel"
        const val NOTIFICATION_ID = 1001
        const val ACTION_START = "ai.hime.mobile.START_SYNC"
        const val ACTION_STOP = "ai.hime.mobile.STOP_SYNC"
    }

    private val serviceScope = CoroutineScope(Dispatchers.IO + Job())
    private lateinit var keystoreManager: KeystoreManager
    private lateinit var postureAuditor: SecurityPostureAuditor
    private lateinit var mtlsClient: MtlsClient
    private var heartbeatJob: Job? = null

    override fun onCreate() {
        super.onCreate()
        keystoreManager = KeystoreManager()
        postureAuditor = SecurityPostureAuditor(this)
        mtlsClient = MtlsClient(keystoreManager)
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> {
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
            }
            else -> {
                val notification = buildForegroundNotification("Hime Companion Active")
                startForeground(NOTIFICATION_ID, notification)
                startHeartbeatReporting()
            }
        }
        return START_STICKY
    }

    private fun startHeartbeatReporting() {
        heartbeatJob?.cancel()
        heartbeatJob = serviceScope.launch {
            while (isActive) {
                reportDeviceStatus()
                delay(60_000) // 1 minute interval
            }
        }
    }

    private fun reportDeviceStatus() {
        val batteryManager = getSystemService(Context.BATTERY_SERVICE) as? BatteryManager
        val batteryPercent = batteryManager?.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY) ?: 50
        val isCharging = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val status = batteryManager?.getIntProperty(BatteryManager.BATTERY_PROPERTY_STATUS)
            status == BatteryManager.BATTERY_STATUS_CHARGING
        } else false

        val statFs = StatFs(filesDir.absolutePath)
        val storageFree = statFs.availableBytes
        val storageTotal = statFs.totalBytes
        val posture = postureAuditor.auditPosture()

        val status = MobileDeviceStatus(
            deviceId = "android-" + Build.MODEL.replace(" ", "-").lowercase(),
            batteryPercent = batteryPercent,
            isCharging = isCharging,
            storageFreeBytes = storageFree,
            storageTotalBytes = storageTotal,
            securityPosture = posture,
            timestamp = System.currentTimeMillis() / 1000
        )

        try {
            val jsonParams = Json.encodeToJsonElement(status).jsonObject
            mtlsClient.sendRpc("device.status.report", jsonParams)
        } catch (_: Exception) {}
    }

    private fun buildForegroundNotification(content: String): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Hime Desktop Companion")
            .setContentText(content)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setOngoing(true)
            .setContentIntent(pendingIntent)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Hime Device Synchronization",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Maintains secure link and background synchronization with Hime Desktop"
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        heartbeatJob?.cancel()
        mtlsClient.disconnect()
        super.onDestroy()
    }
}

