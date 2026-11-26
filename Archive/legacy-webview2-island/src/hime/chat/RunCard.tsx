import React, { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, ChevronDown, ChevronUp, AlertCircle, Clock, Loader2 } from "lucide-react";
import { ChatRun } from "./types";
import { ProgressBar } from "../ui/ProgressBar";
import { Pill } from "../ui/Pill";
import { ChatMessageItem } from "./ChatMessageItem";

export interface RunCardProps {
  run: ChatRun;
  onCancel?: (runId: string) => void;
  onRetry?: (run: ChatRun) => void;
}

/**
 * Process Run Card Component
 * - In-place morphing to ChatMessageItem on status === "completed"
 * - Non-blocking progress pill with snappy spring animation
 * - Targeted cancel button '✕' (<= 10 chars)
 * - Collapsible activity step log ("Lebih" / "Tutup" <= 10 chars)
 * - Strict copy limits: Title <= 16, Desc <= 36, Button <= 10, Error <= 44 chars
 */
export const RunCard: React.FC<RunCardProps> = ({ run, onCancel, onRetry }) => {
  const [showSteps, setShowSteps] = useState(false);

  // In-place morphing when completed
  if (run.status === "completed") {
    return (
      <motion.div
        layout
        initial={{ opacity: 0.8, y: 4 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
        style={{ width: "100%" }}
      >
        <ChatMessageItem
          message={{
            id: `ast_${run.id}`,
            role: "assistant",
            text: run.resultText || "Respons selesai.",
            timestamp: run.completedAt || run.createdAt,
            runId: run.id,
            skills: run.skills
          }}
        />
      </motion.div>
    );
  }

  const isQueued = run.status === "queued";
  const isCancelled = run.status === "cancelled";
  const isError = run.status === "error";

  const queueLabel = run.queueIndex ? `Antre #${run.queueIndex}` : "Antre";
  const truncatedStatus = run.statusText.slice(0, 36);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.98, y: 6 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.96 }}
      transition={{ type: "spring", stiffness: 480, damping: 32 }}
      style={{
        width: "100%",
        maxWidth: "92%",
        margin: "8px 0",
        padding: "10px 12px",
        borderRadius: "14px",
        background: isCancelled
          ? "rgba(27, 30, 38, 0.6)"
          : isError
          ? "rgba(255, 77, 94, 0.08)"
          : "rgba(20, 22, 28, 0.95)",
        border: isCancelled
          ? "1px solid rgba(255, 255, 255, 0.06)"
          : isError
          ? "1px solid rgba(255, 77, 94, 0.25)"
          : "1px solid rgba(52, 227, 166, 0.24)",
        boxSizing: "border-box"
      }}
    >
      {/* Card Header: Status pill, status text, and targeted cancel button */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "8px",
          marginBottom: isQueued ? "4px" : "8px"
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "6px", minWidth: 0, flex: 1 }}>
          {isQueued ? (
            <Pill
              label={queueLabel.slice(0, 16)}
              variant="solid"
              accent="amber"
              icon={<Clock size={11} />}
            />
          ) : isCancelled ? (
            <Pill
              label="Dibatalkan"
              variant="outline"
              accent="red"
            />
          ) : isError ? (
            <Pill
              label="Gagal"
              variant="solid"
              accent="red"
              icon={<AlertCircle size={11} />}
            />
          ) : (
            <Pill
              label="Memproses"
              variant="solid"
              accent="mint"
              icon={<Loader2 size={11} className="hime-spin" />}
            />
          )}

          <span
            style={{
              fontSize: "11px",
              color: isCancelled ? "#6F7785" : "#A7AEBB",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap"
            }}
            title={run.statusText}
          >
            {truncatedStatus}
          </span>
        </div>

        {/* Action button: Cancel '✕' */}
        {(run.status === "running" || run.status === "queued") && onCancel && (
          <button
            type="button"
            onClick={() => onCancel(run.id)}
            title="Batalkan"
            style={{
              background: "rgba(255, 255, 255, 0.06)",
              border: "none",
              borderRadius: "6px",
              width: "22px",
              height: "22px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: "#A7AEBB",
              transition: "transform 160ms ease-out, background 160ms ease-out",
              flexShrink: 0
            }}
            onMouseDown={(e) => (e.currentTarget.style.transform = "scale(0.92)")}
            onMouseUp={(e) => (e.currentTarget.style.transform = "scale(1)")}
          >
            <X size={12} />
          </button>
        )}

        {/* Retry button on error */}
        {isError && onRetry && (
          <button
            type="button"
            onClick={() => onRetry(run)}
            title="Coba lagi"
            style={{
              background: "rgba(255, 77, 94, 0.15)",
              border: "1px solid rgba(255, 77, 94, 0.3)",
              borderRadius: "6px",
              padding: "2px 8px",
              fontSize: "10.5px",
              cursor: "pointer",
              color: "#FF4D5E",
              fontWeight: 500,
              flexShrink: 0
            }}
          >
            Coba lagi
          </button>
        )}
      </div>

      {/* Progress Bar for active running state */}
      {run.status === "running" && (
        <div style={{ margin: "4px 0 8px 0" }}>
          <ProgressBar value={run.progress} accent="mint" showLabel />
        </div>
      )}

      {/* Error message if present */}
      {isError && run.error && (
        <div
          style={{
            fontSize: "11px",
            color: "#FF4D5E",
            margin: "4px 0",
            wordBreak: "break-word"
          }}
        >
          {run.error.slice(0, 44)}
        </div>
      )}

      {/* Activity Step Log Toggle ("Lebih" / "Tutup") */}
      {run.steps && run.steps.length > 0 && (
        <div>
          <button
            type="button"
            onClick={() => setShowSteps((prev) => !prev)}
            style={{
              background: "none",
              border: "none",
              padding: "2px 0",
              fontSize: "10.5px",
              color: "#6F7785",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              gap: "4px",
              transition: "color 0.15s ease"
            }}
          >
            {showSteps ? <ChevronUp size={11} /> : <ChevronDown size={11} />}
            <span>{showSteps ? "Tutup" : "Lebih"}</span>
          </button>

          <AnimatePresence>
            {showSteps && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: "auto", opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.18, ease: "easeOut" }}
                style={{
                  overflow: "hidden",
                  marginTop: "4px",
                  paddingLeft: "8px",
                  borderLeft: "2px solid rgba(255, 255, 255, 0.08)"
                }}
              >
                {run.steps.map((step, idx) => (
                  <div
                    key={idx}
                    style={{
                      fontSize: "10px",
                      color: "#A7AEBB",
                      lineHeight: 1.5,
                      margin: "2px 0"
                    }}
                  >
                    • {step.slice(0, 36)}
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      )}
    </motion.div>
  );
};
