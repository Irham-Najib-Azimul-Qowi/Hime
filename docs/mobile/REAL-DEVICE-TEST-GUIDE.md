# Hime Mobile Real-Device Verification Guide (Owner Gate #6)

> **Purpose:** Runbook for Project Owner to verify Hime Android Companion on a physical Android smartphone or tablet.  
> **Status:** Owner Acceptance Gate Runbook  
> **Target Android Versions:** Android 11 through Android 15

---

## 1. Prerequisites

1. **Physical Android Device:** Android 8.0 (API 26) or higher (recommended: Android 13+).
2. **Wi-Fi Connectivity:** Both Desktop and Android device must be on the same Local Area Network (or connected via a shared Tailscale / WireGuard mesh VPN).
3. **Android Studio / ADB (Optional):** If building directly from source via `gradlew assembleDebug`.

---

## 2. Step-by-Step Verification Procedure

### Test 1: Desktop Pairing Initiation
1. Launch Hime Desktop on Windows.
2. Press `Alt+6` or navigate to the **Devices** tab in the sidebar.
3. Click `+ Pair New Android Device`.
4. **Verification Point:**
   - A modal appears showing the desktop server IP, port `20130`, a single-use pairing token, and a prominent 6-digit confirmation code.
   - An expiration countdown timer (300 seconds) starts running.

---

### Test 2: Mobile App Pairing & Anti-Replay
1. Launch Hime Mobile Companion on the Android device.
2. In the pairing screen:
   - Enter Desktop IP and Port (`20130`).
   - Enter the pairing token and 6-digit confirmation code displayed on Desktop.
   - Tap `Confirm Pairing`.
3. **Verification Point:**
   - The device immediately registers with Hime Core over mTLS.
   - The Desktop UI updates in real-time showing the device with `ACTIVE` status badge and SHA-256 certificate fingerprint.
   - **Negative Verification:** Attempting to pair a second device with the same token fails immediately with *Invalid Token* (anti-replay verified).

---

### Test 3: Remote Action Approval (UR-02 / UR-04)
1. On Desktop, initiate an action requiring approval (e.g., in Omnibox or Hermes conversation: ask to delete a test file).
2. Switch to the Android Companion app and tap the **Tasks** tab.
3. **Verification Point:**
   - The pending approval appears instantly with action type (e.g., `storage.delete_file`), file path, and explanation.
   - Tap `Approve` on the Android screen.
   - The approval resolution transmits securely to Hime Core.
   - On Desktop, the action proceeds immediately.
   - Attempting to tap `Approve` again or resolving from desktop shows the ticket already consumed.

---

### Test 4: Mobile Storage & Resumable File Transfer
1. On Android Companion, switch to the **Storage** tab.
2. Verify total and free storage metrics accurately reflect the device's internal storage.
3. Initiate a file transfer from Desktop to Mobile.
4. **Verification Point:**
   - The transfer progress indicator on Android animates in real-time.
   - Chunks are verified on-the-fly using SHA-256 digests.
   - When complete, the file arrives intact with identical hash.

---

### Test 5: Instant Revocation
1. On Desktop under the **Devices** tab, click `Revoke` on the paired Android device.
2. Confirm the revocation dialog.
3. **Verification Point:**
   - Device status changes to `REVOKED`.
   - Any subsequent WebSocket or RPC request from the Android phone is immediately rejected by Hime Core with `UNAUTHORIZED (Device access revoked)`.

