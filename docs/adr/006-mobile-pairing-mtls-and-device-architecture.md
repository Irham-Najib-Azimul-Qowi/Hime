# ADR 006: Mobile Device Architecture, Mutual TLS Pairing & Remote Control Boundaries

> **Status:** Accepted  
> **Date:** 2026-11-14  
> **Deciders:** Principal Systems Architect, Lead Security Engineer, Owner  
> **Consulted:** Android Security Guidelines, NIST SP 800-207 (Zero Trust)

---

## 1. Context

Hime aims to unify personal desktop and mobile computing into a cohesive AI operating environment. Users need to monitor agent activity, approve high-risk actions remotely (UR-02 / UR-04), converse with Hermes Agent, and inspect phone storage.

However, mobile devices introduce unique security challenges:
- Communication occurs across untrusted local Wi-Fi networks where rogue devices could eavesdrop or attempt MITM injection.
- Unrestricted remote mobile control is either dangerous or impossible under modern Android platform constraints without root access.
- Cloud relay servers would expose private conversation and personal telemetry to third parties.

---

## 2. Decision

### A. Mutual TLS (mTLS) with Local Root CA
We decided that Hime Core will act as an independent Local PKI Certificate Authority. Mobile devices generate ECDSA P-256 keys directly inside `AndroidKeyStore`. During pairing, public keys and fingerprints are exchanged and bound to device records. All subsequent traffic requires mutual TLS authentication.

### B. Two-Way Single-Use Pairing Handshake
Pairing requires physical presence:
1. Desktop generates a short-lived (5-minute TTL) pairing token and 6-digit confirmation code.
2. The user scans the QR code or manually inputs the confirmation code on their phone.
3. The token transitions atomically from `pending` to `consumed` in the SQLite database, preventing replay attacks.

### C. Zero-Root, Official API Compliance
We reject all root exploits, accessibility scraping hacks, and remote shell implementations. The Android companion operates exclusively within official platform APIs:
- File management: Storage Access Framework (SAF) and MediaStore.
- Posture reporting: `KeyguardManager` and `DevicePolicyManager`.
- Background syncing: `ForegroundService` with user-visible notification.

### D. Direct LAN & Private Mesh Connectivity Only
In version 1.0, Hime does not run external cloud relays or open UPnP ports to the public internet. Connectivity is supported over Local Area Networks (LAN) and user-configured private mesh networks (e.g. Tailscale / WireGuard).

---

## 3. Consequences

### Positive
- **Zero Third-Party Dependency:** Complete privacy; no corporate cloud relay sits between desktop and mobile.
- **Fail-Closed Security:** Stolen or leaked tokens cannot be replayed; revoked devices lose access immediately.
- **Play Store Compliance:** The Android companion conforms to Google Play policies with zero risk of malware flags.

### Negative / Trade-offs
- Remote access outside the home Wi-Fi requires users to configure a mesh VPN (e.g., Tailscale).
- Background syncing requires an ongoing notification to avoid Android OEM task killing.

