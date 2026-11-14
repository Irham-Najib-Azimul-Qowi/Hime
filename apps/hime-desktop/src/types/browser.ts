export interface BrowserSession {
  id: string;
  url: string;
  title: string;
  status: 'active' | 'closed';
  started_at: number;
  last_active_at: number;
}

export interface BrowserHistoryItem {
  id: string;
  url: string;
  title: string;
  visited_at: number;
  metadata: string;
}

export interface BrowserDownload {
  id: string;
  source_url: string;
  target_path: string;
  file_size_bytes: number;
  sha256: string;
  quarantined: boolean;
  quarantine_reason: string | null;
  downloaded_at: number;
}

export interface BrowserExtractionResult {
  session_id: string;
  url: string;
  title: string;
  extracted_markdown: string;
  links: string[];
  timestamp: number;
}

