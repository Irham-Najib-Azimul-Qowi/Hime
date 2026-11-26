import { useState, useCallback, useRef } from "react";
import { open as openTauriDialog } from "@tauri-apps/plugin-dialog";
import { AttachmentRef } from "./types";
import { classifyAttachment } from "./skillsCatalog";

export const MAX_ATTACHMENTS = 10;

/**
 * Fast SHA-256 computation using Web Crypto API
 */
export async function computeBufferSha256(buffer: ArrayBuffer): Promise<string> {
  if (typeof crypto !== "undefined" && crypto.subtle) {
    try {
      const hashBuf = await crypto.subtle.digest("SHA-256", buffer);
      const hashArr = Array.from(new Uint8Array(hashBuf));
      return hashArr.map((b) => b.toString(16).padStart(2, "0")).join("");
    } catch {
      // Fallback if subtle crypto is unavailable
    }
  }
  // Deterministic fallback for headless/mock environments
  return "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
}

export function useFileAttachment() {
  const [attachments, setAttachments] = useState<AttachmentRef[]>([]);
  const [isDraggingOver, setIsDraggingOver] = useState(false);
  const [isFileDialogOpen, setIsFileDialogOpen] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const processFile = useCallback(async (file: File): Promise<AttachmentRef> => {
    let sha256 = "";
    try {
      const buffer = await file.arrayBuffer();
      sha256 = await computeBufferSha256(buffer);
    } catch {
      sha256 = "0".repeat(64);
    }

    const { category, accent } = classifyAttachment(file.name, file.type);

    let thumbnailUrl: string | undefined;
    if (file.type.startsWith("image/") && typeof URL !== "undefined") {
      try {
        thumbnailUrl = URL.createObjectURL(file);
      } catch {
        // Ignored in non-browser envs
      }
    }

    let snippet: string | undefined;
    let content: string | undefined;
    const isTextLike =
      file.type.startsWith("text/") ||
      file.type.includes("json") ||
      file.type.includes("javascript") ||
      /\.(txt|md|rs|ts|tsx|js|jsx|json|csv|py|html|css|log|toml|yaml|yml|xml|sh|ps1|bat|ini|conf)$/i.test(file.name);

    if (isTextLike && file.size < 500_000) {
      try {
        const text = await file.text();
        content = text;
        snippet = text.slice(0, 1500);
      } catch {
        // Ignored
      }
    }

    return {
      id: `att_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
      name: file.name,
      size: file.size,
      type: file.type || "application/octet-stream",
      category,
      accent,
      sha256,
      thumbnailUrl,
      snippet,
      content
    };
  }, []);

  const addFiles = useCallback(
    async (newFiles: File[] | FileList) => {
      const list = Array.from(newFiles);
      if (list.length === 0) return;

      const remainingSlots = MAX_ATTACHMENTS - attachments.length;
      if (remainingSlots <= 0) return;

      const toProcess = list.slice(0, remainingSlots);
      const processed = await Promise.all(toProcess.map(processFile));

      setAttachments((prev) => [...prev, ...processed].slice(0, MAX_ATTACHMENTS));
    },
    [attachments.length, processFile]
  );

  const removeAttachment = useCallback((id: string) => {
    setAttachments((prev) => prev.filter((a) => a.id !== id));
  }, []);

  const clearAttachments = useCallback(() => {
    setAttachments([]);
  }, []);

  // Adds files from native file paths (e.g. from Tauri dialog or drag-and-drop events)
  const addFilePaths = useCallback(
    (paths: string[]) => {
      if (!paths || paths.length === 0) return;
      const remainingSlots = MAX_ATTACHMENTS - attachments.length;
      if (remainingSlots <= 0) return;

      const toProcess = paths.slice(0, remainingSlots);
      const newRefs: AttachmentRef[] = toProcess.map((p) => {
        const pathStr = typeof p === "string" ? p : (p as any).path || "";
        const name = pathStr.split(/[\\/]/).pop() || "berkas";
        const { category, accent } = classifyAttachment(name);
        return {
          id: `att_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
          name,
          path: pathStr,
          size: 0,
          type: "application/octet-stream",
          category,
          accent,
          sha256: "0".repeat(64),
          snippet: `[Lokasi: ${pathStr}]`
        };
      });

      setAttachments((prev) => [...prev, ...newRefs].slice(0, MAX_ATTACHMENTS));
    },
    [attachments.length]
  );

  // Native file dialog picker with web fallback & dismissal guard
  const openFilePicker = useCallback(async () => {
    setIsFileDialogOpen(true);
    if (typeof window !== "undefined") {
      (window as any).isFileDialogOpen = true;
      (window as any).__isFileDialogOpen = true;
      (window as any).__himeFileDialogOpen = true;
    }

    const resetDialogGuard = () => {
      setTimeout(() => {
        setIsFileDialogOpen(false);
        if (typeof window !== "undefined") {
          (window as any).isFileDialogOpen = false;
          (window as any).__isFileDialogOpen = false;
          (window as any).__himeFileDialogOpen = false;
        }
      }, 500);
    };

    try {
      if (typeof openTauriDialog === "function") {
        const selected = await openTauriDialog({
          multiple: true,
          directory: false,
          title: "Pilih Berkas Lampiran"
        });
        if (selected) {
          const paths = Array.isArray(selected) ? selected : [selected];
          addFilePaths(paths as string[]);
          resetDialogGuard();
          return;
        }
        resetDialogGuard();
        return;
      }
    } catch {
      // Fallback to DOM input element
    }

    // DOM Fallback
    if (!fileInputRef.current && typeof document !== "undefined") {
      const input = document.createElement("input");
      input.type = "file";
      input.multiple = true;
      input.style.display = "none";
      input.onchange = (e: any) => {
        if (e.target?.files) {
          addFiles(e.target.files);
        }
        resetDialogGuard();
      };
      input.oncancel = () => {
        resetDialogGuard();
      };
      document.body.appendChild(input);
      fileInputRef.current = input;
    }
    fileInputRef.current?.click();

    if (typeof window !== "undefined") {
      window.addEventListener(
        "focus",
        () => {
          resetDialogGuard();
        },
        { once: true }
      );
    }
  }, [addFiles, addFilePaths]);

  // Drag and drop event handlers
  const onDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingOver(true);
  }, []);

  const onDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingOver(false);
  }, []);

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDraggingOver(false);
      if (e.dataTransfer && e.dataTransfer.files) {
        addFiles(e.dataTransfer.files);
      }
    },
    [addFiles]
  );

  // Clipboard paste event handler
  const onPaste = useCallback(
    (e: React.ClipboardEvent) => {
      if (e.clipboardData && e.clipboardData.files && e.clipboardData.files.length > 0) {
        e.preventDefault();
        addFiles(e.clipboardData.files);
      }
    },
    [addFiles]
  );

  return {
    attachments,
    isDraggingOver,
    isFileDialogOpen,
    addFiles,
    addFilePaths,
    removeAttachment,
    clearAttachments,
    openFilePicker,
    onDragOver,
    onDragLeave,
    onDrop,
    onPaste
  };
}
