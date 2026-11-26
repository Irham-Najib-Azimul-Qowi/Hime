export interface GitStatusInfo {
	branch: string;
	ahead: number;
	behind: number;
	staged_count: number;
	unstaged_count: number;
	untracked_count: number;
	is_clean: boolean;
}

export interface CommandRunResult {
	command: string;
	args: string[];
	exit_code: number;
	stdout: string;
	stderr: string;
	duration_ms: number;
	is_success: boolean;
}

export interface SecretFinding {
	file_path: string;
	line_number: number;
	secret_type: string;
	masked_match: string;
}

export const ALLOWED_DEV_BINARIES = [
	"git",
	"cargo",
	"pnpm",
	"npm",
	"node",
	"python",
	"python3",
	"uv",
	"pytest",
	"rustc",
	"tsc",
	"ruff",
	"eslint"
] as const;
