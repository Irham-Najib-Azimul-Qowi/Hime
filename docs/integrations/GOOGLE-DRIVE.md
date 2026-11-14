# Hime Integration: Google Drive Adapter

## 1. Overview & Purpose

The Google Drive Adapter (`hime-core::integrations::drive`) provides secure cloud document synchronization, file discovery, and sharing governance.

Because enterprise and academic workspaces often contain sensitive intellectual property and personal data, external sharing operations through the adapter are strictly gated:

1. **File Search & Listing**: Autonomous read access across designated folders and root directories.
2. **Document Upload**: Backups and generated academic reports can be synced directly to user-selected folders.
3. **Sharing Permission Gate**: Any operation adding a collaborator (`integrations.drive.files.share`) generates a mandatory human approval request containing target file ID, recipient email, and proposed role (`reader`, `commenter`, `editor`).

---

## 2. Security Boundaries & Token Management

- OAuth 2.0 refresh tokens are encrypted using Windows DPAPI.
- File uploads are validated to ensure they originate from within allowed workspace roots or the Hime document sandbox (`.hime/documents_sandbox`).
- Path traversal in remote folder IDs is blocked.

---

## 3. Endpoints

| RPC Endpoint | Purpose | Approval Required |
|:---|:---|:---:|
| `integrations.drive.files.list` | List remote Google Drive files | No |
| `integrations.drive.files.upload` | Upload local file to Google Drive | Yes |
| `integrations.drive.files.share` | Share file or folder with email | Yes |

