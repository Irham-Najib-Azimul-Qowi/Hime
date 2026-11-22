package ai.hime.mobile.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import ai.hime.mobile.ui.theme.*

data class RoutineItem(
    val id: String,
    val title: String,
    val category: String,
    val time: String,
    val durationMin: Int,
    val streak: Int,
    val isActive: Boolean = false
)

@Composable
fun FocusScreen() {
    var isEmergencyPaused by remember { mutableStateOf(false) }
    var pauseRemainingSecs by remember { mutableStateOf(900) } // 15 mins
    var activeTimerSeconds by remember { mutableStateOf(1500) } // 25 mins
    var isTimerRunning by remember { mutableStateOf(false) }

    val routines = remember {
        mutableStateListOf(
            RoutineItem("r1", "Belajar Pemrograman", "Belajar", "19:00", 45, 5, true),
            RoutineItem("r2", "Membaca Buku", "Pengembangan", "21:00", 30, 12, false),
            RoutineItem("r3", "Review Jurnal Harian", "Refleksi", "22:00", 15, 3, false)
        )
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(16.dp)
    ) {
        // Section Header
        Text(
            text = "FOKUS & PENEGAKAN",
            color = VioletAccent,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Sesi & Rutinitas",
            color = TextPrimary,
            fontSize = 20.sp,
            fontWeight = FontWeight.SemiBold
        )

        Spacer(modifier = Modifier.height(16.dp))

        // Active Session Timer Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(16.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = if (isTimerRunning) "SESI BELAJAR AKTIF" else "SIAP MEMULAI SESI",
                    color = if (isTimerRunning) EmeraldSuccess else TextSecondary,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp
                )
                Spacer(modifier = Modifier.height(8.dp))
                val mins = activeTimerSeconds / 60
                val secs = activeTimerSeconds % 60
                Text(
                    text = String.format("%02d:%02d", mins, secs),
                    color = TextPrimary,
                    fontSize = 42.sp,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(12.dp))
                Row(
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Button(
                        onClick = { isTimerRunning = !isTimerRunning },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = if (isTimerRunning) CrimsonDanger else VioletAccent
                        ),
                        shape = RoundedCornerShape(10.dp)
                    ) {
                        Icon(
                            imageVector = if (isTimerRunning) Icons.Default.Pause else Icons.Default.PlayArrow,
                            contentDescription = if (isTimerRunning) "Jeda" else "Mulai",
                            modifier = Modifier.size(16.dp)
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(if (isTimerRunning) "Jeda" else "Mulai")
                    }
                    OutlinedButton(
                        onClick = {
                            isTimerRunning = false
                            activeTimerSeconds = 1500
                        },
                        shape = RoundedCornerShape(10.dp),
                        colors = ButtonDefaults.outlinedButtonColors(contentColor = TextSecondary)
                    ) {
                        Text("Reset")
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Category Usage Bar (Daily Screen Time Breakdown)
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(14.dp)
        ) {
            Column(modifier = Modifier.padding(14.dp)) {
                Text(
                    text = "Penggunaan Hari Ini (UR-11)",
                    color = TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold
                )
                Spacer(modifier = Modifier.height(10.dp))
                // Progress Bar
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(8.dp)
                        .clip(CircleShape)
                ) {
                    Box(modifier = Modifier.weight(0.65f).fillMaxHeight().background(VioletAccent))
                    Box(modifier = Modifier.weight(0.20f).fillMaxHeight().background(Color(0xFF38BDF8)))
                    Box(modifier = Modifier.weight(0.15f).fillMaxHeight().background(Color(0xFFF59E0B)))
                }
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.size(8.dp).background(VioletAccent, CircleShape))
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Belajar (65%)", color = TextSecondary, fontSize = 11.sp)
                    }
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFF38BDF8), CircleShape))
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Netral (20%)", color = TextSecondary, fontSize = 11.sp)
                    }
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFFF59E0B), CircleShape))
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Pengalih (15%)", color = TextSecondary, fontSize = 11.sp)
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Jeda Darurat (Emergency Pause)
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(
                containerColor = if (isEmergencyPaused) Color(0xFF2E1C1C) else ObsidianSurfaceElevated
            ),
            shape = RoundedCornerShape(14.dp)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(14.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = if (isEmergencyPaused) "Jeda Darurat Aktif" else "Jeda Darurat (15 Menit)",
                        color = if (isEmergencyPaused) CrimsonDanger else TextPrimary,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                    Text(
                        text = if (isEmergencyPaused) "Sisa waktu: ${pauseRemainingSecs / 60}m ${pauseRemainingSecs % 60}s" else "Buka penegakan sementara untuk urusan penting.",
                        color = TextMuted,
                        fontSize = 11.sp
                    )
                }
                Button(
                    onClick = {
                        isEmergencyPaused = !isEmergencyPaused
                        if (isEmergencyPaused) pauseRemainingSecs = 900
                    },
                    colors = ButtonDefaults.buttonColors(
                        containerColor = if (isEmergencyPaused) ObsidianSurface else CrimsonDanger
                    ),
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Text(
                        text = if (isEmergencyPaused) "Akhiri" else "Jeda",
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Routines List
        Text(
            text = "Daftar Rutinitas",
            color = TextPrimary,
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold
        )
        Spacer(modifier = Modifier.height(8.dp))

        LazyColumn(
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(routines.size) { index ->
                val r = routines[index]
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
                    shape = RoundedCornerShape(12.dp)
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column {
                            Text(r.title, color = TextPrimary, fontSize = 13.sp, fontWeight = FontWeight.Medium)
                            Text("${r.time} • ${r.durationMin} mnt • Kategori: ${r.category}", color = TextSecondary, fontSize = 11.sp)
                        }
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(Icons.Default.Whatshot, contentDescription = "Streak", tint = Color(0xFFF59E0B), modifier = Modifier.size(16.dp))
                            Spacer(modifier = Modifier.width(3.dp))
                            Text("${r.streak}", color = Color(0xFFF59E0B), fontSize = 12.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                }
            }
        }
    }
}
