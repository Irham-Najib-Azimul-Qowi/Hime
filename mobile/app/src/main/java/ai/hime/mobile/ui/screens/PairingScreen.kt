package ai.hime.mobile.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import ai.hime.mobile.ui.theme.*

@Composable
fun PairingScreen(
    onPairingConfirmed: (String, String, String, Int) -> Unit
) {
    var serverIp by remember { mutableStateOf("192.168.1.100") }
    var serverPort by remember { mutableStateOf("20130") }
    var pairingToken by remember { mutableStateOf("") }
    var confirmationCode by remember { mutableStateOf("") }
    var statusMessage by remember { mutableStateOf<String?>(null) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(ObsidianBackground)
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "HIME COMPANION",
            color = VioletAccent,
            fontSize = 14.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 2.sp
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "Pair with Desktop",
            color = TextPrimary,
            fontSize = 24.sp,
            fontWeight = FontWeight.SemiBold
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "Scan QR Code on Hime Desktop or enter the pairing code and server details manually.",
            color = TextSecondary,
            fontSize = 13.sp,
            lineHeight = 18.sp
        )

        Spacer(modifier = Modifier.height(24.dp))

        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ObsidianSurface),
            shape = RoundedCornerShape(16.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                OutlinedTextField(
                    value = serverIp,
                    onValueChange = { serverIp = it },
                    label = { Text("Desktop IP Address") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(12.dp))
                OutlinedTextField(
                    value = serverPort,
                    onValueChange = { serverPort = it },
                    label = { Text("Port (Default 20130)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(12.dp))
                OutlinedTextField(
                    value = pairingToken,
                    onValueChange = { pairingToken = it },
                    label = { Text("Pairing Token") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(12.dp))
                OutlinedTextField(
                    value = confirmationCode,
                    onValueChange = { if (it.length <= 6) confirmationCode = it },
                    label = { Text("6-Digit Confirmation Code") },
                    placeholder = { Text("e.g. 123456") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        Button(
            onClick = {
                val port = serverPort.toIntOrNull() ?: 20130
                if (pairingToken.isNotBlank() && confirmationCode.length == 6) {
                    onPairingConfirmed(pairingToken, confirmationCode, serverIp, port)
                } else {
                    statusMessage = "Please enter both valid token and 6-digit code"
                }
            },
            colors = ButtonDefaults.buttonColors(containerColor = VioletAccent),
            shape = RoundedCornerShape(12.dp),
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
        ) {
            Text("Confirm Pairing", fontWeight = FontWeight.SemiBold)
        }

        statusMessage?.let { msg ->
            Spacer(modifier = Modifier.height(12.dp))
            Text(msg, color = CrimsonDanger, fontSize = 12.sp)
        }
    }
}

