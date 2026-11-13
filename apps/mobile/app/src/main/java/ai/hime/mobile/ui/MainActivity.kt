package ai.hime.mobile.ui

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import ai.hime.mobile.data.model.ApprovalItem
import ai.hime.mobile.data.model.SecurityPosture
import ai.hime.mobile.data.security.SecurityPostureAuditor
import ai.hime.mobile.service.HimeDeviceForegroundService
import ai.hime.mobile.ui.screens.*
import ai.hime.mobile.ui.theme.*

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Start background sync service
        val serviceIntent = Intent(this, HimeDeviceForegroundService::class.java).apply {
            action = HimeDeviceForegroundService.ACTION_START
        }
        startService(serviceIntent)

        val auditor = SecurityPostureAuditor(this)
        val posture = auditor.auditPosture()

        setContent {
            HimeTheme {
                var isPaired by remember { mutableStateOf(false) }
                var currentTab by remember { mutableStateOf(0) }

                val pendingApprovals = remember {
                    mutableStateListOf(
                        ApprovalItem(
                            id = "ticket-001",
                            actionType = "storage.delete_file",
                            path = "C:/Users/User/Downloads/large_dump.zip",
                            description = "Delete obsolete 4GB archive",
                            expiresAt = System.currentTimeMillis() + 300_000
                        )
                    )
                }

                if (!isPaired) {
                    PairingScreen(
                        onPairingConfirmed = { token, code, ip, port ->
                            isPaired = true
                        }
                    )
                } else {
                    Scaffold(
                        bottomBar = {
                            NavigationBar(
                                containerColor = ObsidianSurface,
                                contentColor = TextPrimary
                            ) {
                                NavigationBarItem(
                                    selected = currentTab == 0,
                                    onClick = { currentTab = 0 },
                                    icon = { Icon(Icons.Default.Chat, contentDescription = "Chat") },
                                    label = { Text("Chat") }
                                )
                                NavigationBarItem(
                                    selected = currentTab == 1,
                                    onClick = { currentTab = 1 },
                                    icon = { Icon(Icons.Default.Assignment, contentDescription = "Tasks") },
                                    label = { Text("Tasks") }
                                )
                                NavigationBarItem(
                                    selected = currentTab == 2,
                                    onClick = { currentTab = 2 },
                                    icon = { Icon(Icons.Default.Storage, contentDescription = "Storage") },
                                    label = { Text("Storage") }
                                )
                                NavigationBarItem(
                                    selected = currentTab == 3,
                                    onClick = { currentTab = 3 },
                                    icon = { Icon(Icons.Default.Security, contentDescription = "Security") },
                                    label = { Text("Security") }
                                )
                                NavigationBarItem(
                                    selected = currentTab == 4,
                                    onClick = { currentTab = 4 },
                                    icon = { Icon(Icons.Default.Settings, contentDescription = "Settings") },
                                    label = { Text("Settings") }
                                )
                            }
                        }
                    ) { innerPadding ->
                        Surface(
                            modifier = Modifier
                                .fillMaxSize()
                                .padding(innerPadding),
                            color = ObsidianBackground
                        ) {
                            when (currentTab) {
                                0 -> ChatScreen(onSendMessage = {})
                                1 -> TaskBoardScreen(
                                    approvals = pendingApprovals,
                                    onResolveApproval = { ticketId, decision ->
                                        // Resolution handled by server
                                    }
                                )
                                2 -> StorageScreen()
                                3 -> SecurityScreen(posture = posture)
                                4 -> SettingsScreen(onDisconnect = { isPaired = false })
                            }
                        }
                    }
                }
            }
        }
    }
}
