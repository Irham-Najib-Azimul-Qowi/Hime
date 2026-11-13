package ai.hime.mobile.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import ai.hime.mobile.ui.theme.*

@Composable
fun StorageScreen(
    totalBytes: Long = 128_000_000_000L,
    freeBytes: Long = 45_000_000_000L,
    transferProgress: Float? = null // 0.0 to 1.0 or null
) {
    val usedBytes = totalBytes - freeBytes
    val usedGb = "%.1f".format(usedBytes / 1_000_000_000.0)
    val totalGb = "%.1f".format(totalBytes / 1_000_000_000.0)
    val usedRatio = (usedBytes.toFloat() / totalBytes.toFloat()).coerceIn(0f, 1f)

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(16.dp)
    ) {
        Text(
            text = "STORAGE INTELLIGENCE",
            color = VioletAccent,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Phone Storage Status",
            color = TextPrimary,
            fontSize = 20.sp,
            fontWeight = FontWeight.SemiBold
        )

        Spacer(modifier = Modifier.height(16.dp))

        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(14.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text("Internal Storage", color = TextSecondary, fontSize = 13.sp)
                Spacer(modifier = Modifier.height(4.dp))
                Text("$usedGb GB used of $totalGb GB", color = TextPrimary, fontSize = 18.sp, fontWeight = FontWeight.Bold)

                Spacer(modifier = Modifier.height(12.dp))
                LinearProgressIndicator(
                    progress = { usedRatio },
                    color = VioletAccent,
                    trackColor = ObsidianBorder,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(8.dp)
                )

                Spacer(modifier = Modifier.height(12.dp))
                Text(
                    text = "Boundaries: Protected by Storage Access Framework (SAF). Content is never uploaded without explicit sync approval.",
                    color = TextMuted,
                    fontSize = 11.sp,
                    lineHeight = 16.sp
                )
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Active transfers section
        Text(
            text = "Cross-Device File Transfers",
            color = TextPrimary,
            fontSize = 16.sp,
            fontWeight = FontWeight.SemiBold
        )
        Spacer(modifier = Modifier.height(8.dp))

        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(14.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                if (transferProgress != null) {
                    Text("Transfer in progress: report_2026.pdf", color = TextPrimary, fontSize = 14.sp)
                    Spacer(modifier = Modifier.height(8.dp))
                    LinearProgressIndicator(
                        progress = { transferProgress },
                        color = EmeraldSuccess,
                        trackColor = ObsidianBorder,
                        modifier = Modifier.fillMaxWidth().height(6.dp)
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text("${(transferProgress * 100).toInt()}% • Verified with SHA-256 chunks", color = TextSecondary, fontSize = 11.sp)
                } else {
                    Text("No active transfers. Ready for incoming or outgoing sync.", color = TextSecondary, fontSize = 13.sp)
                }
            }
        }
    }
}
