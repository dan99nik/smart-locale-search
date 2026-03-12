import type { SearchResult } from "../lib/types";
import { FileIcon } from "./FileIcon";
import { SnippetPreview } from "./SnippetPreview";
import { openFile, openFileAtLine, revealInFolder, getImageThumbnail } from "../lib/tauri";
import { useCallback, useEffect, useState } from "react";

const CODE_EXTENSIONS = new Set([
  "js","ts","py","cs","cpp","c","go","rs","java","jsx","tsx","h","hpp",
  "rb","sh","yaml","yml","toml","xml","sql","html","css","json",
]);

const IMAGE_EXTENSIONS = new Set(["jpg", "jpeg", "png", "webp"]);

interface ResultItemProps {
  result: SearchResult;
  selected: boolean;
  onSelect: () => void;
  query?: string;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(isoString: string): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays}d ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
    return date.toLocaleDateString();
  } catch {
    return isoString;
  }
}

function matchBadge(matchType: string): { label: string; color: string } {
  switch (matchType) {
    case "exact+semantic":
      return { label: "Exact + AI", color: "bg-accent/20 text-accent" };
    case "semantic":
      return { label: "AI match", color: "bg-purple-500/20 text-purple-400" };
    case "image_semantic":
      return { label: "Visual AI", color: "bg-violet-500/20 text-violet-400" };
    case "exact":
      return { label: "Exact", color: "bg-success/20 text-success" };
    case "content":
      return { label: "Content", color: "bg-blue-500/20 text-blue-400" };
    case "filename":
      return { label: "Name", color: "bg-yellow-500/20 text-yellow-400" };
    default:
      return { label: "Fuzzy", color: "bg-gray-500/20 text-gray-400" };
  }
}

function isCodeFile(ext: string | null): boolean {
  return ext != null && CODE_EXTENSIONS.has(ext.toLowerCase());
}

function isImageFile(ext: string | null): boolean {
  return ext != null && IMAGE_EXTENSIONS.has(ext.toLowerCase());
}

function ImageThumbnail({ path }: { path: string }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getImageThumbnail(path)
      .then((data) => { if (!cancelled) setSrc(data); })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [path]);

  if (!src) {
    return (
      <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-pink-500/10 shrink-0">
        <span className="text-xs font-bold text-pink-400">IMG</span>
      </div>
    );
  }

  return (
    <img
      src={src}
      alt=""
      className="w-10 h-10 rounded-lg object-cover shrink-0 bg-bg-tertiary"
    />
  );
}

function shortenPath(path: string): string {
  const home = path.replace(/^\/Users\/[^/]+/, "~");
  if (home.length <= 60) return home;
  const parts = home.split("/");
  if (parts.length <= 3) return home;
  return `${parts[0]}/.../${parts.slice(-2).join("/")}`;
}

export function ResultItem({ result, selected, onSelect, query }: ResultItemProps) {
  const badge = matchBadge(result.match_type);
  const codeFile = isCodeFile(result.extension);
  const imageFile = isImageFile(result.extension);
  const hasLineInfo = result.line_start != null;

  const handleOpen = useCallback(() => {
    if (hasLineInfo && result.line_start != null) {
      openFileAtLine(result.path, result.line_start);
    } else {
      openFile(result.path);
    }
  }, [result.path, result.line_start, hasLineInfo]);

  const handleReveal = useCallback(() => {
    revealInFolder(result.path);
  }, [result.path]);

  const handleCopyPath = useCallback(() => {
    navigator.clipboard.writeText(result.path);
  }, [result.path]);

  return (
    <div
      className={`group px-5 py-3 cursor-pointer transition-colors border-b border-border/50
        ${selected ? "bg-bg-active" : "hover:bg-bg-hover"}`}
      onClick={onSelect}
      onDoubleClick={handleOpen}
    >
      <div className="flex items-start gap-3">
        {imageFile ? (
          <ImageThumbnail path={result.path} />
        ) : (
          <FileIcon extension={result.extension} />
        )}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-text-primary truncate">
              {result.filename}
            </span>
            <span
              className={`text-[10px] font-medium px-1.5 py-0.5 rounded-full ${badge.color}`}
            >
              {badge.label}
            </span>
            {codeFile && (
              <span className="text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400">
                Code
              </span>
            )}
            {imageFile && result.match_type === "image_semantic" && (
              <span className="text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-violet-500/15 text-violet-400">
                Visual
              </span>
            )}
            {imageFile && result.match_type !== "image_semantic" && (
              <span className="text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-pink-500/15 text-pink-400">
                OCR
              </span>
            )}
          </div>

          {(result.symbol_name || hasLineInfo) && (
            <div className="flex items-center gap-2 mt-0.5">
              {result.symbol_name && (
                <span className="text-xs font-mono text-accent truncate">
                  {result.symbol_name}
                </span>
              )}
              {hasLineInfo && (
                <span className="text-[11px] text-text-muted font-mono">
                  L{result.line_start}
                  {result.line_end && result.line_end !== result.line_start
                    ? `–${result.line_end}`
                    : ""}
                </span>
              )}
            </div>
          )}

          <p className="text-xs text-text-muted truncate mt-0.5">
            {shortenPath(result.path)}
          </p>
          {result.snippet && <SnippetPreview snippet={result.snippet} query={query} />}
          <div className="flex items-center gap-3 mt-1.5">
            <span className="text-[11px] text-text-muted">
              {formatSize(result.size)}
            </span>
            <span className="text-[11px] text-text-muted">
              {formatDate(result.modified_time)}
            </span>
            <div className="flex-1" />
            <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleOpen();
                }}
                className="text-[11px] px-2 py-0.5 rounded bg-bg-tertiary hover:bg-accent text-text-secondary hover:text-white transition-colors"
              >
                {hasLineInfo ? `Open :${result.line_start}` : "Open"}
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleReveal();
                }}
                className="text-[11px] px-2 py-0.5 rounded bg-bg-tertiary hover:bg-accent text-text-secondary hover:text-white transition-colors"
              >
                Reveal
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleCopyPath();
                }}
                className="text-[11px] px-2 py-0.5 rounded bg-bg-tertiary hover:bg-accent text-text-secondary hover:text-white transition-colors"
              >
                Copy path
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
