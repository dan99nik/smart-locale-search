const ICON_MAP: Record<string, { icon: string; color: string }> = {
  pdf: { icon: "PDF", color: "#ef4444" },
  docx: { icon: "DOC", color: "#3b82f6" },
  doc: { icon: "DOC", color: "#3b82f6" },
  csv: { icon: "CSV", color: "#22c55e" },
  txt: { icon: "TXT", color: "#9ca3af" },
  md: { icon: "MD", color: "#9ca3af" },
  js: { icon: "JS", color: "#eab308" },
  jsx: { icon: "JSX", color: "#eab308" },
  ts: { icon: "TS", color: "#3b82f6" },
  tsx: { icon: "TSX", color: "#3b82f6" },
  py: { icon: "PY", color: "#3b82f6" },
  rs: { icon: "RS", color: "#f97316" },
  cpp: { icon: "C++", color: "#6366f1" },
  cs: { icon: "C#", color: "#8b5cf6" },
  json: { icon: "{ }", color: "#6b7280" },
  html: { icon: "< >", color: "#f97316" },
  css: { icon: "CSS", color: "#3b82f6" },
  jpg: { icon: "IMG", color: "#ec4899" },
  jpeg: { icon: "IMG", color: "#ec4899" },
  png: { icon: "IMG", color: "#ec4899" },
  webp: { icon: "IMG", color: "#ec4899" },
};

interface FileIconProps {
  extension: string | null;
  className?: string;
}

export function FileIcon({ extension, className = "" }: FileIconProps) {
  const ext = extension?.toLowerCase() ?? "";
  const mapped = ICON_MAP[ext] ?? { icon: "FILE", color: "#6b7280" };

  return (
    <div
      className={`flex items-center justify-center w-10 h-10 rounded-lg text-xs font-bold shrink-0 ${className}`}
      style={{ backgroundColor: `${mapped.color}20`, color: mapped.color }}
    >
      {mapped.icon}
    </div>
  );
}
