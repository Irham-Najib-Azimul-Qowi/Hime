import React, { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";
import { Copy, Check, FileText } from "lucide-react";
import { ChatMessage } from "./types";
import { Pill } from "../ui/Pill";
import { formatFileSize } from "./skillsCatalog";

interface CodeBlockProps {
  language?: string;
  value: string;
}

/**
 * CodeBlock with language indicator and 1-click clipboard copy
 * Button label: "Salin" / "Tersalin" (<= 10 chars)
 */
export const CodeBlock: React.FC<CodeBlockProps> = ({ language, value }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    if (typeof navigator !== "undefined" && navigator.clipboard) {
      navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div
      style={{
        margin: "8px 0",
        borderRadius: "8px",
        background: "rgba(10, 10, 14, 0.94)",
        border: "1px solid rgba(255, 255, 255, 0.1)",
        overflow: "hidden"
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "4px 10px",
          background: "rgba(255, 255, 255, 0.04)",
          borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
          fontSize: "10px",
          color: "#94a3b8",
          fontFamily: "monospace"
        }}
      >
        <span>{language || "code"}</span>
        <button
          type="button"
          onClick={handleCopy}
          style={{
            background: "none",
            border: "none",
            color: copied ? "#34E3A6" : "#94a3b8",
            cursor: "pointer",
            fontSize: "10px",
            display: "flex",
            alignItems: "center",
            gap: "4px",
            padding: "2px 6px",
            borderRadius: "4px",
            transition: "all 0.15s ease"
          }}
          title={copied ? "Tersalin" : "Salin"}
        >
          {copied ? <Check size={11} /> : <Copy size={11} />}
          <span>{copied ? "Tersalin" : "Salin"}</span>
        </button>
      </div>
      <pre
        style={{
          margin: 0,
          padding: "10px",
          overflowX: "auto",
          fontSize: "11px",
          lineHeight: 1.5,
          fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
          color: "#e2e8f0"
        }}
      >
        <code>{value}</code>
      </pre>
    </div>
  );
};

export interface ChatMessageItemProps {
  message: ChatMessage;
}

export const ChatMessageItem: React.FC<ChatMessageItemProps> = ({ message }) => {
  const isUser = message.role === "user";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: isUser ? "flex-end" : "flex-start",
        marginBottom: "12px",
        width: "100%",
        boxSizing: "border-box"
      }}
    >
      {/* Skill or Context metadata */}
      {message.skills && message.skills.length > 0 && (
        <div style={{ marginBottom: "4px", display: "flex", gap: "4px" }}>
          {message.skills.map((skill) => (
            <Pill key={skill} label={`/${skill}`} variant="tint" accent="lime" />
          ))}
        </div>
      )}

      {/* Main message bubble */}
      <div
        style={{
          maxWidth: "88%",
          borderRadius: isUser ? "14px 14px 2px 14px" : "14px 14px 14px 2px",
          padding: isUser ? "8px 12px" : "10px 14px",
          background: isUser ? "rgba(52, 227, 166, 0.12)" : "rgba(20, 22, 28, 0.95)",
          border: isUser
            ? "1px solid rgba(52, 227, 166, 0.28)"
            : "1px solid rgba(255, 255, 255, 0.08)",
          color: "#f4f6fa",
          fontSize: "12px",
          lineHeight: 1.5,
          wordBreak: "break-word"
        }}
      >
        {isUser ? (
          <div>{message.text}</div>
        ) : (
          <ReactMarkdown
            remarkPlugins={[remarkGfm, remarkMath]}
            rehypePlugins={[rehypeKatex]}
            components={{
              code({ className, children, ...props }: any) {
                const match = /language-(\w+)/.exec(className || "");
                const textVal = String(children).replace(/\n$/, "");
                if (match || textVal.includes("\n")) {
                  return <CodeBlock language={match ? match[1] : ""} value={textVal} />;
                }
                return (
                  <code
                    style={{
                      background: "rgba(255, 255, 255, 0.08)",
                      padding: "1px 5px",
                      borderRadius: "4px",
                      fontSize: "11px",
                      fontFamily: "monospace",
                      color: "#34E3A6"
                    }}
                    {...props}
                  >
                    {children}
                  </code>
                );
              },
              p({ children }) {
                return <p style={{ margin: "4px 0", lineHeight: 1.55 }}>{children}</p>;
              },
              ul({ children }) {
                return <ul style={{ paddingLeft: "16px", margin: "4px 0" }}>{children}</ul>;
              },
              ol({ children }) {
                return <ol style={{ paddingLeft: "16px", margin: "4px 0" }}>{children}</ol>;
              },
              li({ children }) {
                return <li style={{ margin: "2px 0" }}>{children}</li>;
              },
              table({ children }) {
                return (
                  <div style={{ overflowX: "auto", margin: "6px 0" }}>
                    <table style={{ borderCollapse: "collapse", width: "100%", fontSize: "11px" }}>
                      {children}
                    </table>
                  </div>
                );
              },
              th({ children }) {
                return (
                  <th
                    style={{
                      borderBottom: "1px solid rgba(255, 255, 255, 0.16)",
                      padding: "4px 8px",
                      textAlign: "left",
                      color: "#e2e8f0",
                      fontWeight: 600
                    }}
                  >
                    {children}
                  </th>
                );
              },
              td({ children }) {
                return (
                  <td
                    style={{
                      borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
                      padding: "4px 8px",
                      color: "#cbd5e1"
                    }}
                  >
                    {children}
                  </td>
                );
              }
            }}
          >
            {message.text}
          </ReactMarkdown>
        )}

        {/* Render Attachments if present */}
        {message.attachments && message.attachments.length > 0 && (
          <div
            style={{
              marginTop: "8px",
              paddingTop: "6px",
              borderTop: "1px solid rgba(255, 255, 255, 0.06)",
              display: "flex",
              flexWrap: "wrap",
              gap: "6px"
            }}
          >
            {message.attachments.map((att) => (
              <div
                key={att.id}
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "4px",
                  padding: "2px 8px",
                  borderRadius: "6px",
                  background: "rgba(255, 255, 255, 0.05)",
                  border: "1px solid rgba(255, 255, 255, 0.1)",
                  fontSize: "10.5px"
                }}
              >
                <FileText size={11} color={`var(--accent-${att.accent})`} />
                <span
                  style={{
                    maxWidth: "110px",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap"
                  }}
                >
                  {att.name}
                </span>
                <span style={{ color: "#6F7785", fontSize: "9.5px" }}>
                  ({formatFileSize(att.size)})
                </span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Hover timestamp */}
      <span
        style={{
          fontSize: "9.5px",
          color: "#6F7785",
          marginTop: "3px",
          padding: "0 4px"
        }}
      >
        {new Date(message.timestamp).toLocaleTimeString([], {
          hour: "2-digit",
          minute: "2-digit"
        })}
      </span>
    </div>
  );
};
