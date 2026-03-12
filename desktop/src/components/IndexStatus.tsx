import type { IndexStats } from "../lib/types";

interface IndexStatusProps {
  stats: IndexStats | null;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function IndexStatus({ stats }: IndexStatusProps) {
  if (!stats) return null;

  return (
    <div className="flex items-center gap-4 px-5 py-2 border-t border-border bg-bg-secondary text-xs text-text-muted">
      <span>{stats.total_files.toLocaleString()} files indexed</span>
      <span className="text-border">|</span>
      <span>{stats.total_chunks.toLocaleString()} chunks</span>
      {stats.total_vectors > 0 && (
        <>
          <span className="text-border">|</span>
          <span>{stats.total_vectors.toLocaleString()} vectors</span>
        </>
      )}
      <span className="text-border">|</span>
      <span>{stats.total_roots} folder{stats.total_roots !== 1 ? "s" : ""}</span>
      <div className="flex-1" />
      <span>{formatBytes(stats.db_size_bytes)} DB</span>
    </div>
  );
}
