import { PillAccent } from "../ui/Pill";
export type { EyeExpression } from "../ui/HimeEyes";

/**
 * Status lifecycle of an AI Process Run
 */
export type RunStatus = "queued" | "running" | "completed" | "error" | "cancelled";

/**
 * Categorization of file attachments mapping to design tokens
 */
export type AttachmentCategory =
  | "dev"
  | "doc"
  | "data"
  | "image"
  | "archive"
  | "audio"
  | "video"
  | "other";

/**
 * Metadata reference for an uploaded or attached file
 */
export interface AttachmentRef {
  id: string;
  name: string;
  path?: string;
  size: number;
  type: string;
  category: AttachmentCategory;
  accent: PillAccent;
  sha256: string;
  thumbnailUrl?: string;
  snippet?: string;
  content?: string;
}

/**
 * Multi-run active or queued execution unit
 */
export interface ChatRun {
  id: string;
  prompt: string;
  skills: string[];
  attachments: AttachmentRef[];
  status: RunStatus;
  queueIndex?: number; // e.g. 1 for "Antre #1"
  progress: number; // 0 to 100
  statusText: string; // Hard constraint: <= 36 characters
  steps: string[]; // Activity log items
  resultText?: string;
  error?: string; // Hard constraint: <= 44 characters
  createdAt: number;
  completedAt?: number;
}

/**
 * Chat message model in the single global thread
 */
export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  text: string;
  timestamp: number;
  runId?: string;
  skills?: string[];
  attachments?: AttachmentRef[];
  isSteering?: boolean;
}

/**
 * 9 Canonical Skill Categories per 07-CHAT-SPEC.md §3 and 04-DESIGN-SYSTEM.md
 */
export type SkillCategory =
  | "tasks"
  | "time"
  | "storage"
  | "docs"
  | "dev"
  | "web"
  | "translate"
  | "device"
  | "system";

/**
 * Canonical skill definition
 */
export interface SkillManifest {
  id: string;
  slash: string; // e.g. "/dev", "/doc"
  name: string; // Hard constraint: <= 16 characters
  desc: string; // Hard constraint: <= 36 characters
  category: SkillCategory;
  accent: PillAccent;
  iconName: string;
  aliases?: string[];
}

/**
 * Fast control command definition
 */
export interface FastCommand {
  command: "/stop" | "/jeda" | "/cari" | "/bersih" | string;
  name: string; // Hard constraint: <= 16 characters
  desc: string; // Hard constraint: <= 36 characters
}
