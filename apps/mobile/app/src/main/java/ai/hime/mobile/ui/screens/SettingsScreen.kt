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
fun SettingsScreen(
    serverAddress: String = "192.168.1.100:20130",
    onDisconnect: () -> Unit
) {
    var usageStatsOptIn by remember { mutableStateOf(false) }
    var strictnessMode by remember { mutableStateOf("Balanced") }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(16.dp)
    ) {
        Text(
            text = "PREFERENCES & PRIVACY",
            color = VioletAccent,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Companion Settings",
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
                Text("Paired Desktop Gateway", color = TextSecondary, fontSize = 12.sp)
                Text(serverAddress, color = TextPrimary, fontSize = 15.sp, fontWeight = FontWeight.Bold)

                Divider(color = ObsidianBorder, modifier = Modifier.padding(vertical = 12.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("App Usage Nudges (UR-11)", color = TextPrimary, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                        Text("Opt-in screen time nudges for distracting apps.", color = TextMuted, fontSize = 11.sp)
                    }
                    Switch(
                        checked = usageStatsOptIn,
                        onCheckedChange = { usageStatsOptIn = it },
                        colors = SwitchDefaults.colors(checkedThumbColor = VioletAccent)
                    )
                }

                Divider(color = ObsidianBorder, modifier = Modifier.padding(vertical = 12.dp))

                Text("Strictness Mode", color = TextPrimary, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    listOf("Relaxed", "Balanced", "Strict").forEach { mode ->
                        val isSelected = strictnessMode == mode
                        Button(
                            onClick = { strictnessMode = mode },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = if (isSelected) VioletAccent else ObsidianSurfaceElevated
                            ),
                            shape = RoundedCornerShape(8.dp),
                            modifier = Modifier.weight(1f)
                        ) {
                            Text(mode, fontSize = 12.sp)
                        }
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        OutlinedButton(
            onClick = onDisconnect,
            colors = ButtonDefaults.outlinedButtonColors(contentColor = CrimsonDanger),
            shape = RoundedCornerShape(12.dp),
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
        ) {
            Text("Unpair from Hime Desktop", fontWeight = FontWeight.SemiBold)
        }
    }
}

