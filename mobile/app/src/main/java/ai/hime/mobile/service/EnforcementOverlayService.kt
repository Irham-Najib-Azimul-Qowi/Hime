package ai.hime.mobile.service

import android.app.Service
import android.content.Context
import android.content.Intent
import android.graphics.PixelFormat
import android.os.Build
import android.os.IBinder
import android.provider.Settings
import android.view.Gravity
import android.view.LayoutInflater
import android.view.View
import android.view.WindowManager
import android.widget.Button
import android.widget.TextView

/**
 * Tier B Enforcement Overlay Service (16-MOBILE-PARITY.md §4).
 * Uses SYSTEM_ALERT_WINDOW to present a full-screen distraction guard
 * with Hime mascot eyes, countdown timer, and emergency pause button.
 */
class EnforcementOverlayService : Service() {

    private var windowManager: WindowManager? = null
    private var overlayView: View? = null

    companion object {
        const val EXTRA_APP_NAME = "extra_app_name"
        const val EXTRA_REMAINING_SECS = "extra_remaining_secs"

        fun start(context: Context, appName: String, remainingSecs: Int = 300) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M && !Settings.canDrawOverlays(context)) {
                return
            }
            val intent = Intent(context, EnforcementOverlayService::class.java).apply {
                putExtra(EXTRA_APP_NAME, appName)
                putExtra(EXTRA_REMAINING_SECS, remainingSecs)
            }
            context.startService(intent)
        }

        fun stop(context: Context) {
            val intent = Intent(context, EnforcementOverlayService::class.java)
            context.stopService(intent)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val appName = intent?.getStringExtra(EXTRA_APP_NAME) ?: "Aplikasi Pengalih"
        val remainingSecs = intent?.getIntExtra(EXTRA_REMAINING_SECS, 300) ?: 300

        showOverlay(appName, remainingSecs)
        return START_NOT_STICKY
    }

    private fun showOverlay(appName: String, remainingSecs: Int) {
        if (overlayView != null) return

        windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager

        val layoutParamsType = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
        } else {
            @Suppress("DEPRECATION")
            WindowManager.LayoutParams.TYPE_PHONE
        }

        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.MATCH_PARENT,
            WindowManager.LayoutParams.MATCH_PARENT,
            layoutParamsType,
            WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL or
                    WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN or
                    WindowManager.LayoutParams.FLAG_FULLSCREEN,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.CENTER
        }

        // Programmatic overlay view to prevent layout XML inflation dependencies
        val root = android.widget.LinearLayout(this).apply {
            orientation = android.widget.LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setBackgroundColor(android.graphics.Color.parseColor("#EE0D0F14")) // Dark obsidian
            setPadding(48, 48, 48, 48)

            // Mascot eye indicator
            addView(TextView(context).apply {
                text = "( > ⏝ < )"
                textSize = 28f
                setTextColor(android.graphics.Color.parseColor("#F87171")) // Red / upset
                gravity = Gravity.CENTER
            })

            // Title
            addView(TextView(context).apply {
                text = "Waktu Belajar Aktif"
                textSize = 20f
                setTypeface(null, android.graphics.Typeface.BOLD)
                setTextColor(android.graphics.Color.parseColor("#F3F4F6"))
                setPadding(0, 24, 0, 8)
                gravity = Gravity.CENTER
            })

            // Message
            addView(TextView(context).apply {
                text = "Sudah melebihi batas waktu di $appName.\nKembali fokus ke rutinitas belajarmu."
                textSize = 14f
                setTextColor(android.graphics.Color.parseColor("#9CA3AF"))
                gravity = Gravity.CENTER
                setPadding(0, 0, 0, 32)
            })

            // Action Buttons Container
            addView(android.widget.LinearLayout(context).apply {
                orientation = android.widget.LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER

                // Jeda Darurat Button
                addView(Button(context).apply {
                    text = "Jeda (15m)"
                    setBackgroundColor(android.graphics.Color.parseColor("#1F2937"))
                    setTextColor(android.graphics.Color.parseColor("#F87171"))
                    setOnClickListener {
                        stopSelf()
                    }
                })

                // Spacer
                addView(View(context).apply {
                    layoutParams = android.widget.LinearLayout.LayoutParams(24, 1)
                })

                // Kembali Belajar Button
                addView(Button(context).apply {
                    text = "Kembali Belajar"
                    setBackgroundColor(android.graphics.Color.parseColor("#8B5CF6"))
                    setTextColor(android.graphics.Color.WHITE)
                    setOnClickListener {
                        val homeIntent = Intent(Intent.ACTION_MAIN).apply {
                            addCategory(Intent.CATEGORY_HOME)
                            flags = Intent.FLAG_ACTIVITY_NEW_TASK
                        }
                        startActivity(homeIntent)
                        stopSelf()
                    }
                })
            })
        }

        overlayView = root
        try {
            windowManager?.addView(overlayView, params)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (overlayView != null) {
            windowManager?.removeView(overlayView)
            overlayView = null
        }
    }
}
