export interface DocumentJob {
  id: string;
  template_id: string;
  target_format: string;
  output_path: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  error_message: string | null;
  created_at: number;
  completed_at: number | null;
}

export interface ArtcraftDiscoveryStatus {
  wordcraft_available: boolean;
  figcraft_available: boolean;
  wordcraft_path: string | null;
  figcraft_path: string | null;
  supported_interchanges: string[];
}

export interface DocumentVerificationReport {
  job_id: string;
  output_path: string;
  file_size_bytes: number;
  page_count: number;
  has_cover: boolean;
  has_toc: boolean;
  typography: string;
  margins: string;
  line_spacing: string;
  visual_fidelity_score: number;
  verified_at: number;
}
