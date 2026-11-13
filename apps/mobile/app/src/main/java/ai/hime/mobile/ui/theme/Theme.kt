package ai.hime.mobile.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

val ObsidianBackground = Color(0xFF0D0F14)
val ObsidianSurface = Color(0xFF161922)
val ObsidianSurfaceElevated = Color(0xFF1E222E)
val ObsidianBorder = Color(0xFF2B3142)
val VioletAccent = Color(0xFF8B5CF6)
val VioletAccentHover = Color(0xFF7C3AED)
val EmeraldSuccess = Color(0xFF10B981)
val CrimsonDanger = Color(0xFFEF4444)
val TextPrimary = Color(0xFFF3F4F6)
val TextSecondary = Color(0xFF9CA3AF)
val TextMuted = Color(0xFF6B7280)

private val DarkColorScheme = darkColorScheme(
    primary = VioletAccent,
    onPrimary = Color.White,
    background = ObsidianBackground,
    surface = ObsidianSurface,
    onSurface = TextPrimary,
    surfaceVariant = ObsidianSurfaceElevated,
    onSurfaceVariant = TextSecondary,
    outline = ObsidianBorder
)

@Composable
fun HimeTheme(
    darkTheme: Boolean = true, // Hime obsidian theme is default
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        content = content
    )
}
