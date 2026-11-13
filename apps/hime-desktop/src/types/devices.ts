export interface PairedDevice {
  id: string;
  device_name: string;
  device_type: string;
  cert_fingerprint: string;
  paired_at: number;
  last_seen_at: number;
  status: 'active' | 'revoked';
}

export interface PairingSession {
  server_address: string;
  server_port: number;
  pairing_token: string;
  confirmation_code: string;
  ca_fingerprint: string;
  expires_at: number;
}

export interface DeviceStorageSummary {
  device_id: string;
  total_bytes: number;
  free_bytes: number;
  indexed_items_count: number;
  updated_at: number;
}

export interface FileTransfer {
  id: string;
  device_id: string;
  file_name: string;
  file_size: number;
  bytes_transferred: number;
  direction: 'incoming' | 'outgoing';
  status: 'pending' | 'transferring' | 'completed' | 'failed';
  sha256: string;
  started_at: number;
  completed_at?: number;
}
