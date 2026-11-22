export interface DocumentVerificationReport {
	page_count: number;
	font_family: string;
	has_table_of_contents: boolean;
	has_overflow: boolean;
	is_verified: boolean;
}

export interface DocumentOpResult {
	output_path: string;
	file_size_bytes: number;
	format: string;
	verification: DocumentVerificationReport;
}

export interface DocumentJobItem {
	id: string;
	job_type: string;
	title: string;
	source_path?: string;
	destination_path: string;
	status: "pending" | "processing" | "completed" | "failed";
	created_at: number;
	completed_at?: number | null;
}

export interface ArtcraftDiscoveryInfo {
	is_installed: boolean;
	executable_path?: string | null;
	version?: string | null;
	can_integrate: boolean;
}
