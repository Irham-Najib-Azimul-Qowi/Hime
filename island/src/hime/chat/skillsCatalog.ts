import { SkillCategory, SkillManifest, FastCommand, AttachmentCategory } from "./types";
import { PillAccent } from "../ui/Pill";

/**
 * 9 Canonical Skill Categories mapping to Design System Color Tokens (§2.2 & 07-CHAT-SPEC)
 */
export const CATEGORY_ACCENT_MAP: Record<SkillCategory, PillAccent> = {
  tasks: "sky",
  time: "amber",
  storage: "mint",
  docs: "violet",
  dev: "lime",
  web: "cyan",
  translate: "pink",
  device: "coral",
  system: "text-3"
};

/**
 * Canonical Skill Catalog — Strict Text Length Limits:
 * - name: <= 16 chars
 * - desc: <= 36 chars
 */
export const SKILL_CATALOG: SkillManifest[] = [
  {
    id: "dev",
    slash: "/dev",
    name: "Dev & Kode",
    desc: "Coding, software, git, & debug",
    category: "dev",
    accent: "lime",
    iconName: "Code",
    aliases: ["kode", "program", "rust", "ts"]
  },
  {
    id: "doc",
    slash: "/doc",
    name: "Dokumen & Teks",
    desc: "Format naskah & karya ilmiah",
    category: "docs",
    accent: "violet",
    iconName: "FileText",
    aliases: ["makalah", "skripsi", "naskah", "laporan"]
  },
  {
    id: "data",
    slash: "/data",
    name: "Data & Analisis",
    desc: "Statistik dataset CSV & audit",
    category: "web",
    accent: "cyan",
    iconName: "BarChart3",
    aliases: ["dataset", "analisis", "csv"]
  },
  {
    id: "simpan",
    slash: "/simpan",
    name: "File & Storage",
    desc: "Organisasi berkas & unduhan",
    category: "storage",
    accent: "mint",
    iconName: "Archive",
    aliases: ["berkas", "unduhan", "clean"]
  },
  {
    id: "tugas",
    slash: "/tugas",
    name: "Tugas & Todo",
    desc: "Kelola agenda dan komitmen",
    category: "tasks",
    accent: "sky",
    iconName: "CheckSquare",
    aliases: ["todo", "agenda", "jadwal"]
  },
  {
    id: "rutin",
    slash: "/rutin",
    name: "Waktu & Fokus",
    desc: "Penegakan fokus dan timer",
    category: "time",
    accent: "amber",
    iconName: "Clock",
    aliases: ["fokus", "timer", "pomodoro"]
  },
  {
    id: "terjemah",
    slash: "/terjemah",
    name: "Terjemahan",
    desc: "Alih bahasa cepat & akurat",
    category: "translate",
    accent: "pink",
    iconName: "Languages",
    aliases: ["translate", "kamus", "arti"]
  },
  {
    id: "perangkat",
    slash: "/perangkat",
    name: "Perangkat",
    desc: "Sinkronisasi Android & laptop",
    category: "device",
    accent: "coral",
    iconName: "Smartphone",
    aliases: ["hp", "android", "device"]
  },
  {
    id: "sistem",
    slash: "/sistem",
    name: "Sistem & PC",
    desc: "Cek CPU, RAM, memori & daya",
    category: "system",
    accent: "text-3",
    iconName: "Cpu",
    aliases: ["sys", "ram", "cpu", "baterai"]
  }
];

/**
 * Fast Control Commands intercepted directly by Chat Engine (07-CHAT-SPEC §3)
 * Limits:
 * - name: <= 16 chars
 * - desc: <= 36 chars
 */
export const FAST_COMMANDS: FastCommand[] = [
  {
    command: "/stop",
    name: "Hentikan Run",
    desc: "Hentikan proses yang berjalan"
  },
  {
    command: "/jeda",
    name: "Jeda Darurat",
    desc: "Jeda rutinitas dan alarm"
  },
  {
    command: "/cari",
    name: "Cari Pesan",
    desc: "Cari riwayat percakapan"
  },
  {
    command: "/bersih",
    name: "Bersih Layar",
    desc: "Sembunyikan pesan di layar"
  }
];

/**
 * Extension to Attachment Category & Accent mapper
 */
export function classifyAttachment(
  filename: string,
  mimeType?: string
): { category: AttachmentCategory; accent: PillAccent } {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  const mime = (mimeType || "").toLowerCase();

  // Code / Dev
  if (
    [
      "rs",
      "ts",
      "tsx",
      "js",
      "jsx",
      "py",
      "go",
      "c",
      "cpp",
      "h",
      "json",
      "html",
      "css",
      "toml",
      "yaml",
      "yml",
      "sql",
      "sh",
      "ps1",
      "bat"
    ].includes(ext) ||
    mime.includes("javascript") ||
    mime.includes("typescript") ||
    mime.includes("json")
  ) {
    return { category: "dev", accent: "lime" };
  }

  // Documents
  if (
    ["pdf", "doc", "docx", "odt", "txt", "md", "rtf", "tex", "ppt", "pptx"].includes(ext) ||
    mime.includes("pdf") ||
    mime.includes("document") ||
    mime.includes("text")
  ) {
    return { category: "doc", accent: "violet" };
  }

  // Data / AI
  if (
    ["csv", "tsv", "xlsx", "xls", "parquet", "sqlite", "db", "arrow"].includes(ext) ||
    mime.includes("csv") ||
    mime.includes("spreadsheet")
  ) {
    return { category: "data", accent: "cyan" };
  }

  // Images
  if (
    ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "avif", "ico"].includes(ext) ||
    mime.startsWith("image/")
  ) {
    return { category: "image", accent: "pink" };
  }

  // Archives
  if (["zip", "tar", "gz", "7z", "rar", "bz2", "xz"].includes(ext) || mime.includes("zip") || mime.includes("compressed")) {
    return { category: "archive", accent: "mint" };
  }

  // Audio
  if (["mp3", "wav", "ogg", "flac", "m4a", "aac", "opus"].includes(ext) || mime.startsWith("audio/")) {
    return { category: "audio", accent: "amber" };
  }

  // Video
  if (["mp4", "mkv", "webm", "mov", "avi"].includes(ext) || mime.startsWith("video/")) {
    return { category: "video", accent: "coral" };
  }

  // Fallback
  return { category: "other", accent: "text-3" };
}

/**
 * Format raw byte size to compact human-readable string (<= 10 chars)
 */
export function formatFileSize(bytes: number): string {
  if (bytes <= 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Format full SHA-256 string to compact preview (8 chars)
 */
export function formatSha256Short(hash: string): string {
  if (!hash) return "";
  return hash.slice(0, 8);
}
