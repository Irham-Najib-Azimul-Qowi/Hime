package ai.hime.mobile.service

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.AccessibilityServiceInfo
import android.content.Context
import android.content.SharedPreferences
import android.view.accessibility.AccessibilityEvent

/**
 * Tier C Enforcement: Accessibility Home Fallback Service (16-MOBILE-PARITY.md §4).
 * Monitors foreground package transitions and redirects to Home when a distracting app
 * is opened during an active enforcement session.
 *
 * Privacy Guarantees:
 * - canRetrieveWindowContent = false (never reads text, fields, or inputs)
 * - Only inspects packageName
 * - Exempts all emergency, phone, maps, and system packages
 */
class HimeAccessibilityEnforcementService : AccessibilityService() {

    companion object {
        var isServiceRunning = false
            private set

        /**
         * Protected package list that can NEVER be blocked or redirected
         */
        val PROTECTED_PACKAGES = setOf(
            "com.android.phone",
            "com.android.dialer",
            "com.google.android.dialer",
            "com.android.emergency",
            "com.android.contacts",
            "com.google.android.contacts",
            "com.google.android.apps.maps",
            "com.android.systemui",
            "ai.hime.mobile"
        )
    }

    private lateinit var prefs: SharedPreferences

    override fun onServiceConnected() {
        super.onServiceConnected()
        isServiceRunning = true
        prefs = getSharedPreferences("hime_enforcement_prefs", Context.MODE_PRIVATE)

        serviceInfo = AccessibilityServiceInfo().apply {
            eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
            feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
            flags = AccessibilityServiceInfo.DEFAULT
            canRetrieveWindowContent = false
            notificationTimeout = 100
        }
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event?.eventType != AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED) return

        val pkgName = event.packageName?.toString() ?: return

        // 1. Protected Apps Exemption
        if (PROTECTED_PACKAGES.contains(pkgName)) {
            return
        }

        // 2. Emergency Pause Exemption
        val isEmergencyPaused = prefs.getBoolean("emergency_pause_active", false)
        val pauseUntil = prefs.getLong("emergency_pause_until", 0L)
        if (isEmergencyPaused && System.currentTimeMillis() < pauseUntil) {
            return
        }

        // 3. Check if enforcement session is currently active
        val isEnforcementActive = prefs.getBoolean("enforcement_session_active", false)
        val blockedApps = prefs.getStringSet("blocked_packages", emptySet()) ?: emptySet()

        if (isEnforcementActive && blockedApps.contains(pkgName)) {
            // Tier C Action: Perform Home navigation smoothly
            performGlobalAction(GLOBAL_ACTION_HOME)

            // Optionally trigger Tier B overlay to explain why
            EnforcementOverlayService.start(applicationContext, pkgName, 300)
        }
    }

    override fun onInterrupt() {
        // Accessibility interrupted
    }

    override fun onDestroy() {
        super.onDestroy()
        isServiceRunning = false
    }
}
