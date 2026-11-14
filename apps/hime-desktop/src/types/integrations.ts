export interface IntegrationAccount {
  provider: 'github' | 'google_drive' | string;
  account_id: string;
  display_name: string;
  scopes: string;
  connected_at: number;
  last_used_at: number;
}

export interface GitHubRepoSummary {
  owner: string;
  repo: string;
  default_branch: string;
  open_issues_count: number;
  open_pr_count: number;
  is_private: boolean;
  description: string;
}

export interface GitHubIssue {
  id: number;
  title: string;
  state: string;
  created_at: string;
  html_url: string;
}

export interface GoogleDriveFile {
  id: string;
  name: string;
  mime_type: string;
  size_bytes: number;
  modified_at: string;
}

