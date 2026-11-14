export interface ResourceBudget {
  max_tokens: number;
  timeout_secs: number;
  max_concurrency: number;
}

export interface SpecialistDefinition {
  id: string;
  name: string;
  role_title: string;
  domain: string;
  responsibilities: string[];
  system_prompt: string;
  allowed_tools: string[];
  max_permission_level: string;
  resource_budget: ResourceBudget;
  is_active: boolean;
  evaluation_score: number;
  version: string;
}

export interface DelegationResult {
  delegation_id: string;
  specialist_id: string;
  task_id: string;
  status: 'running' | 'completed' | 'failed';
  output_summary: string;
  evidence: string[];
  tokens_used: number;
  duration_ms: number;
}

export interface ColumnProfile {
  name: string;
  inferred_type: string;
  missing_count: number;
  missing_percentage: number;
  min: number | null;
  max: number | null;
  mean: number | null;
}

export interface DatasetProfile {
  row_count: number;
  column_count: number;
  columns: ColumnProfile[];
  summary_markdown: string;
}

export interface ExperimentDesign {
  target_variable: string;
  problem_type: string;
  baseline_models: string[];
  recommended_metrics: string[];
  validation_strategy: string;
  uncertainty_and_limitations: string;
  execution_constraints: string;
  model_card_markdown: string;
}

export interface ProactiveSuggestionItem {
  id: string;
  title: string;
  content: string;
  category: string;
  status: 'pending' | 'accepted' | 'dismissed' | 'muted';
  score: number;
  created_at: number;
  responded_at: number | null;
}

export interface ProactiveSettings {
  is_enabled: boolean;
  cooldown_minutes: number;
  muted_until: number | null;
  updated_at: number;
}

export interface EvaluationReport {
  specialist_id: string;
  scenarios_evaluated: number;
  scenarios_passed: number;
  pass_rate: number;
  average_tokens: number;
  average_duration_ms: number;
  overall_score: number;
  evaluation_summary: string;
}
