export interface DeveloperWorkspace {
  id: string;
  name: string;
  root_path: string;
  is_active: boolean;
  registered_at: number;
}

export interface DeveloperCommandRun {
  id: string;
  workspace_id: string;
  command: string;
  args: string;
  exit_code: number;
  duration_ms: number;
  requires_approval: boolean;
  approval_id: string | null;
  run_at: number;
}

export interface DeveloperCommandOutput {
  command_id: string;
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
  truncated: boolean;
}

export interface GitStatusResult {
  branch: string;
  staged: string[];
  modified: string[];
  untracked: string[];
  is_clean: boolean;
}

export interface SeTemplateItem {
  id: string;
  title: string;
  description: string;
  standard: string;
  category: string;
}

