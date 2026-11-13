package ai.hime.mobile.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import ai.hime.mobile.data.model.SecurityPosture
import ai.hime.mobile.ui.theme.*

@Composable
fun SecurityScreen(
    posture: SecurityPosture
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(16.dp)
    ) {
        Text(
            text = "DEVICE INTEGRITY (UR-12)",
            color = VioletAccent,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Security Posture Audit",
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
                SecurityItem(
                    label = "Screen Lock Protection",
                    status = if (posture.isDeviceSecure) "SECURED" else "VULNERABLE",
                    isGood = posture.isDeviceSecure,
                    detail = "Biometrics or PIN/Password configured via KeyguardManager"
                )
                Divider(color = ObsidianBorder, modifier = Modifier.padding(vertical = 12.dp))

                SecurityItem(
                    label = "Storage Encryption",
                    status = if (posture.storageEncrypted) "ENCRYPTED" else "UNENCRYPTED",
                    isGood = posture.storageEncrypted,
                    detail = "Hardware-backed file-based encryption (FBE)"
                )
                Divider(color = ObsidianBorder, modifier = Modifier.padding(vertical = 12.dp))

                SecurityItem(
                    label = "Security Patch Level",
                    status = posture.securityPatchLevel,
                    isGood = true,
                    detail = "Official Android security patch timestamp"
                )
                Divider(color = ObsidianBorder, modifier = Modifier.padding(vertical = 12.dp))

                SecurityItem(
                    label = "Unknown App Installation",
                    status = if (posture.unknownSourcesEnabled) "ENABLED (WARNING)" else "DISABLED (SAFE)",
                    isGood = !posture.unknownSourcesEnabled,
                    detail = "Installation of sideloaded APKs outside verified stores"
                )
                Divider(color = ObsidianBorder, modifier = Modifier.padding(vertical = 12.dp))

                SecurityItem(
                    label = "Operating System Version",
                    status = posture.osVersion,
                    isGood = true,
                    detail = "Current platform runtime"
                )
            }
        }

        Spacer(modifier = Modifier.height(16.dp))
        Text(
            text = "Note: Hime audits device posture using read-only official Android APIs. Hime never modifies OS security configurations or requests root access.",
            color = TextMuted,
            fontSize = 11.sp,
            lineHeight = 16.sp
        )
    }
}

@Composable
private fun SecurityItem(
    label: String,
    status: String,
    isGood: Boolean,
    detail: String
) {
    Column {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(label, color = TextPrimary, fontSize = 14.sp, fontWeight = FontWeight.Medium)
            Text(
                text = status,
                color = if (isGood) EmeraldSuccess else CrimsonDanger,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold
            )
        }
        Spacer(modifier = Modifier.height(2.dp))
        Text(detail, color = TextSecondary, fontSize = 11.sp)
    }
}

