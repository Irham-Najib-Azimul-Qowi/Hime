export interface IndexedFile {
  path: string;
  parent_dir: string;
  file_name: string;
  extension: string | null;
  size_bytes: number;
  modified_at: number;
  created_at: number;
  is_dir: boolean;
  content_hash: string | null;
  mime_type: string | null;
  indexed_at: number;
}

export interface ExtensionBreakdown {
  extension: string;
  count: number;
  total_bytes: number;
}

export interface CleanableSuggestion {
  category: string;
  path: string;
  size_bytes: number;
  reason: string;
}

export interface StorageAnalysisReport {
  base_report: {
    total_files: number;
    total_bytes: number;
    by_extension: ExtensionBreakdown[];
    largest_files: IndexedFile[];
    old_downloads: IndexedFile[];
  };
  cleanable_suggestions: CleanableSuggestion[];
  cleanable_bytes: number;
}

export interface DuplicateGroup {
  content_hash: string;
  size_bytes: number;
  files: IndexedFile[];
}

export interface FileOperation {
  id: string;
  op_type: string;
  source_path: string;
  destination_path: string | null;
  backup_path: string | null;
  source_hash: string | null;
  dest_hash: string | null;
  actor: string;
  approval_id: string | null;
  status: string;
  started_at: number;
  completed_at: number | null;
  error_message: string | null;
}

export interface FileRule {
  id: string;
  name: string;
  pattern: string;
  target_dir: string;
  naming_template: string | null;
  action: string;
  is_active: boolean;
  created_at: number;
  updated_at: number;
}

