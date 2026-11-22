package ai.hime.mobile.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import ai.hime.mobile.ui.theme.*

data class ShelfItem(
    val id: String,
    val type: String, // "text", "file", "url"
    val content: String,
    val preview: String,
    val timestamp: Long = System.currentTimeMillis(),
    val isPinned: Boolean = false,
    val isSynced: Boolean = false
)

@Composable
fun ShelfScreen(
    initialSharedText: String? = null,
    onSendToLaptop: (ShelfItem) -> Unit = {}
) {
    var newClipText by remember { mutableStateOf(initialSharedText ?: "") }
    val shelfItems = remember {
        mutableStateListOf(
            ShelfItem(
                id = "s-1",
                type = "url",
                content = "https://github.com/NousResearch/Hermes-3-Llama-3.1-8B",
                preview = "Repositori Model Hermes 3 (Local AI)",
                isPinned = true,
                isSynced = true
            ),
            ShelfItem(
                id = "s-2",
                type = "text",
                content = "Catatan bab 4: Arsitektur dual-control plane 9Router dan Hime.",
                preview = "Catatan bab 4: Arsitektur dual-control...",
                isPinned = false,
                isSynced = true
            ),
            ShelfItem(
                id = "s-3",
                type = "file",
                content = "/sdcard/Documents/laporan-skpl-revisi.pdf",
                preview = "laporan-skpl-revisi.pdf (2.4 MB)",
                isPinned = false,
                isSynced = false
            )
        )
    }

    LaunchedEffect(initialSharedText) {
        if (!initialSharedText.isNullOrBlank()) {
            newClipText = initialSharedText
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(16.dp)
    ) {
        // Header
        Text(
            text = "SIMPANAN & TRANSFER",
            color = VioletAccent,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Simpanan Cepat",
            color = TextPrimary,
            fontSize = 20.sp,
            fontWeight = FontWeight.SemiBold
        )

        Spacer(modifier = Modifier.height(14.dp))

        // Quick Stash Input (Bagikan ke Hime receiver)
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(14.dp)
        ) {
            Column(modifier = Modifier.padding(12.dp)) {
                OutlinedTextField(
                    value = newClipText,
                    onValueChange = { newClipText = it },
                    placeholder = { Text("Tulis catatan atau tautan...", color = TextMuted, fontSize = 13.sp) },
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = VioletAccent,
                        unfocusedBorderColor = ObsidianBorder,
                        focusedTextColor = TextPrimary,
                        unfocusedTextColor = TextPrimary
                    ),
                    maxLines = 3
                )
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "Bagikan teks/file dari app lain",
                        color = TextMuted,
                        fontSize = 11.sp
                    )
                    Button(
                        onClick = {
                            if (newClipText.isNotBlank()) {
                                val item = ShelfItem(
                                    id = "s-${System.currentTimeMillis()}",
                                    type = if (newClipText.startsWith("http")) "url" else "text",
                                    content = newClipText.trim(),
                                    preview = newClipText.trim().take(36)
                                )
                                shelfItems.add(0, item)
                                newClipText = ""
                            }
                        },
                        colors = ButtonDefaults.buttonColors(containerColor = VioletAccent),
                        shape = RoundedCornerShape(8.dp)
                    ) {
                        Icon(Icons.Default.Add, contentDescription = "Simpan", modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Simpan", fontSize = 12.sp)
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Shelf Items List
        Text(
            text = "Koleksi Simpanan (${shelfItems.size})",
            color = TextPrimary,
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold
        )
        Spacer(modifier = Modifier.height(8.dp))

        LazyColumn(
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            items(shelfItems.size) { index ->
                val item = shelfItems[index]
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = ObsidianSurfaceElevated),
                    shape = RoundedCornerShape(12.dp)
                ) {
                    Column(modifier = Modifier.padding(12.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                val icon = when (item.type) {
                                    "url" -> Icons.Default.Link
                                    "file" -> Icons.Default.AttachFile
                                    else -> Icons.Default.Notes
                                }
                                Icon(icon, contentDescription = item.type, tint = VioletAccent, modifier = Modifier.size(18.dp))
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = item.preview,
                                    color = TextPrimary,
                                    fontSize = 13.sp,
                                    fontWeight = FontWeight.Medium
                                )
                            }
                            if (item.isPinned) {
                                Icon(Icons.Default.PushPin, contentDescription = "Disematkan", tint = Color(0xFFF59E0B), modifier = Modifier.size(14.dp))
                            }
                        }

                        Spacer(modifier = Modifier.height(8.dp))

                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = if (item.isSynced) "Tersinkron ke Core" else "Lokal (Antre)",
                                color = if (item.isSynced) EmeraldSuccess else TextMuted,
                                fontSize = 11.sp
                            )
                            Button(
                                onClick = { onSendToLaptop(item) },
                                colors = ButtonDefaults.buttonColors(containerColor = ObsidianSurface),
                                shape = RoundedCornerShape(6.dp),
                                contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp)
                            ) {
                                Icon(Icons.Default.SendToMobile, contentDescription = "Kirim ke Laptop", tint = VioletAccent, modifier = Modifier.size(14.dp))
                                Spacer(modifier = Modifier.width(4.dp))
                                Text("Ke Laptop", color = TextPrimary, fontSize = 11.sp)
                            }
                        }
                    }
                }
            }
        }
    }
}
