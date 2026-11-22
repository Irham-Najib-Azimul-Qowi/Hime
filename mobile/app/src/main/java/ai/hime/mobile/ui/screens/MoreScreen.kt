package ai.hime.mobile.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import ai.hime.mobile.data.model.SecurityPosture
import ai.hime.mobile.service.OemReliabilityHelper
import ai.hime.mobile.ui.theme.*

@Composable
fun MoreScreen(
    posture: SecurityPosture,
    onDisconnect: () -> Unit
) {
    val context = LocalContext.current
    var selectedSubSection by remember { mutableStateOf<String?>(null) }

    if (selectedSubSection == "storage") {
        Column(modifier = Modifier.fillMaxSize().background(ObsidianBackground)) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = { selectedSubSection = null }) {
                    Icon(Icons.Default.ArrowBack, contentDescription = "Kembali", tint = TextPrimary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Text("Penyimpanan Ponsel", color = TextPrimary, fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
            StorageScreen()
        }
        return
    }

    if (selectedSubSection == "security") {
        Column(modifier = Modifier.fillMaxSize().background(ObsidianBackground)) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = { selectedSubSection = null }) {
                    Icon(Icons.Default.ArrowBack, contentDescription = "Kembali", tint = TextPrimary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Text("Postur Keamanan", color = TextPrimary, fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
            SecurityScreen(posture = posture)
        }
        return
    }

    if (selectedSubSection == "settings") {
        Column(modifier = Modifier.fillMaxSize().background(ObsidianBackground)) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = { selectedSubSection = null }) {
                    Icon(Icons.Default.ArrowBack, contentDescription = "Kembali", tint = TextPrimary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Text("Pengaturan Hime", color = TextPrimary, fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
            SettingsScreen(onDisconnect = onDisconnect)
        }
        return
    }

    // Main More Overview
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(16.dp)
    ) {
        Text(
            text = "PERANGKAT & LAINNYA",
            color = VioletAccent,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Menu Lainnya",
            color = TextPrimary,
            fontSize = 20.sp,
            fontWeight = FontWeight.SemiBold
        )

        Spacer(modifier = Modifier.height(16.dp))

        // Paired Laptop Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(14.dp)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Default.Laptop, contentDescription = "Laptop", tint = VioletAccent, modifier = Modifier.size(24.dp))
                    Spacer(modifier = Modifier.width(12.dp))
                    Column {
                        Text("Laptop Utama (Satoshi)", color = TextPrimary, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                        Text("192.168.1.100:20130 • Tersambung", color = EmeraldSuccess, fontSize = 12.sp)
                    }
                }
                Box(
                    modifier = Modifier
                        .size(10.dp)
                        .background(EmeraldSuccess, shape = RoundedCornerShape(5.dp))
                )
            }
        }

        Spacer(modifier = Modifier.height(14.dp))

        // OEM Battery Optimization Alert / Guide Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurfaceElevated),
            shape = RoundedCornerShape(14.dp)
        ) {
            Column(modifier = Modifier.padding(14.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Default.BatteryChargingFull, contentDescription = "Baterai", tint = Color(0xFFF59E0B), modifier = Modifier.size(18.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Optimasi Baterai & Latar Belakang", color = TextPrimary, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                }
                Spacer(modifier = Modifier.height(6.dp))
                Text(
                    text = OemReliabilityHelper.getOemGuidanceText(),
                    color = TextSecondary,
                    fontSize = 11.sp,
                    lineHeight = 15.sp
                )
                Spacer(modifier = Modifier.height(10.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Button(
                        onClick = { OemReliabilityHelper.openAutostartSettings(context) },
                        colors = ButtonDefaults.buttonColors(containerColor = VioletAccent),
                        shape = RoundedCornerShape(8.dp),
                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp)
                    ) {
                        Text("Buka Autostart", fontSize = 11.sp)
                    }
                    OutlinedButton(
                        onClick = { OemReliabilityHelper.requestIgnoreBatteryOptimizations(context) },
                        shape = RoundedCornerShape(8.dp),
                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp),
                        colors = ButtonDefaults.outlinedButtonColors(contentColor = TextPrimary)
                    ) {
                        Text("Bebaskan Baterai", fontSize = 11.sp)
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Navigation Menu List
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(14.dp)
        ) {
            Column {
                // Storage
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(14.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(Icons.Default.Storage, contentDescription = "Storage", tint = TextSecondary, modifier = Modifier.size(18.dp))
                        Spacer(modifier = Modifier.width(10.dp))
                        Text("Penyimpanan & Berkas HP", color = TextPrimary, fontSize = 13.sp)
                    }
                    Button(
                        onClick = { selectedSubSection = "storage" },
                        colors = ButtonDefaults.buttonColors(containerColor = ObsidianSurfaceElevated),
                        shape = RoundedCornerShape(6.dp),
                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp)
                    ) {
                        Text("Buka", fontSize = 11.sp, color = TextPrimary)
                    }
                }

                Divider(color = ObsidianBorder)

                // Security Posture
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(14.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(Icons.Default.Security, contentDescription = "Security", tint = TextSecondary, modifier = Modifier.size(18.dp))
                        Spacer(modifier = Modifier.width(10.dp))
                        Text("Postur Keamanan & mTLS", color = TextPrimary, fontSize = 13.sp)
                    }
                    Button(
                        onClick = { selectedSubSection = "security" },
                        colors = ButtonDefaults.buttonColors(containerColor = ObsidianSurfaceElevated),
                        shape = RoundedCornerShape(6.dp),
                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp)
                    ) {
                        Text("Lihat", fontSize = 11.sp, color = TextPrimary)
                    }
                }

                Divider(color = ObsidianBorder)

                // Companion Settings
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(14.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(Icons.Default.Settings, contentDescription = "Settings", tint = TextSecondary, modifier = Modifier.size(18.dp))
                        Spacer(modifier = Modifier.width(10.dp))
                        Text("Pengaturan Tambahan", color = TextPrimary, fontSize = 13.sp)
                    }
                    Button(
                        onClick = { selectedSubSection = "settings" },
                        colors = ButtonDefaults.buttonColors(containerColor = ObsidianSurfaceElevated),
                        shape = RoundedCornerShape(6.dp),
                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp)
                    ) {
                        Text("Atur", fontSize = 11.sp, color = TextPrimary)
                    }
                }
            }
        }
    }
}
