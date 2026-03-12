import type { IndexingEvent, IndexingMode, IndexStats } from "../lib/types";

interface IndexingPanelProps {
  indexing: boolean;
  event: IndexingEvent | null;
  mode: IndexingMode;
  error: string | null;
  stats: IndexStats | null;
  onStartIndexing: () => void;
  onCancelIndexing: () => void;
  onSetMode: (mode: IndexingMode) => void;
  embedded?: boolean;
}

export function IndexingPanel({
  indexing,
  event,
  mode,
  error,
  stats,
  onStartIndexing,
  onCancelIndexing,
  onSetMode,
  embedded = false,
}: IndexingPanelProps) {
  const isCompleted = event?.phase === "completed";
  const isScanning = event?.phase === "scanning";
  const hasFiles = stats && stats.total_files > 0;
  const showPanel = embedded || indexing || !hasFiles || isCompleted;

  if (!showPanel) return null;

  return (
    <div className={`rounded-xl border border-border bg-bg-secondary overflow-hidden ${embedded ? "" : "mx-4 mt-3 mb-1"}`}>
      {/* Header row */}
      <div className="flex items-center gap-3 px-4 py-2.5">
        <div className="flex items-center gap-2 flex-1 min-w-0">
          {indexing && (
            <div className="w-3.5 h-3.5 border-2 border-accent border-t-transparent rounded-full animate-spin shrink-0" />
          )}
          {!indexing && !hasFiles && (
            <svg
              className="w-4 h-4 text-accent shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 00-1.883 2.542l.857 6a2.25 2.25 0 002.227 1.932H19.05a2.25 2.25 0 002.227-1.932l.857-6a2.25 2.25 0 00-1.883-2.542m-16.5 0V6A2.25 2.25 0 016 3.75h3.879a1.5 1.5 0 011.06.44l2.122 2.12a1.5 1.5 0 001.06.44H18A2.25 2.25 0 0120.25 9v.776"
              />
            </svg>
          )}
          <span className="text-xs text-text-secondary truncate">
            {indexing
              ? event?.status || "Starting indexing…"
              : isCompleted
                ? event?.status || "Indexing complete"
                : hasFiles
                  ? `${stats!.total_files.toLocaleString()} files indexed`
                  : "Files not indexed yet"}
          </span>
        </div>

        {/* Mode selector */}
        <div className="flex items-center bg-bg-primary rounded-lg border border-border text-[10px] shrink-0">
          <button
            onClick={() => onSetMode("priority")}
            className={`px-2 py-1 rounded-l-lg transition-colors ${
              mode === "priority"
                ? "bg-accent text-white"
                : "text-text-muted hover:text-text-primary"
            }`}
          >
            Priority
          </button>
          <button
            onClick={() => onSetMode("idle")}
            className={`px-2 py-1 rounded-r-lg transition-colors ${
              mode === "idle"
                ? "bg-accent text-white"
                : "text-text-muted hover:text-text-primary"
            }`}
          >
            Idle only
          </button>
        </div>

        {indexing ? (
          <button
            onClick={onCancelIndexing}
            className="text-xs font-medium px-3 py-1 bg-bg-tertiary hover:bg-danger/20 text-text-secondary hover:text-danger rounded-lg transition-colors shrink-0"
          >
            Stop
          </button>
        ) : (
          <button
            onClick={onStartIndexing}
            className="text-xs font-medium px-3 py-1 bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors shrink-0"
          >
            {hasFiles ? "Reindex" : "Start Indexing"}
          </button>
        )}
      </div>

      {/* Progress bar */}
      {indexing && (
        <div className="px-4 pb-2.5">
          <div className="flex items-center gap-2">
            <div className="flex-1 h-1.5 bg-bg-tertiary rounded-full overflow-hidden">
              {isScanning ? (
                <div className="h-full bg-accent rounded-full animate-pulse w-full opacity-40" />
              ) : (
                <div
                  className="h-full bg-accent rounded-full transition-all duration-500 ease-out"
                  style={{ width: `${event?.percent ?? 0}%` }}
                />
              )}
            </div>
            <span className="text-[10px] text-text-muted tabular-nums shrink-0 w-16 text-right">
              {isScanning
                ? "Scanning…"
                : event
                  ? `${Math.round(event.percent)}%`
                  : "0%"}
            </span>
          </div>
          {event && !isScanning && event.total > 0 && (
            <div className="flex items-center gap-3 mt-1">
              <span className="text-[10px] text-text-muted">
                {event.processed.toLocaleString()} / {event.total.toLocaleString()} files
              </span>
              {event.skipped > 0 && (
                <span className="text-[10px] text-text-muted">
                  {event.skipped.toLocaleString()} unchanged
                </span>
              )}
              {event.indexed > 0 && (
                <span className="text-[10px] text-accent">
                  {event.indexed.toLocaleString()} indexed
                </span>
              )}
              {event.errors > 0 && (
                <span className="text-[10px] text-danger">
                  {event.errors.toLocaleString()} errors
                </span>
              )}
            </div>
          )}
        </div>
      )}

      {/* Completed summary */}
      {!indexing && isCompleted && event && (event.skipped > 0 || event.indexed > 0) && (
        <div className="px-4 pb-2.5 flex items-center gap-3">
          {event.indexed > 0 && (
            <span className="text-[10px] text-accent">
              {event.indexed.toLocaleString()} indexed
            </span>
          )}
          {event.skipped > 0 && (
            <span className="text-[10px] text-text-muted">
              {event.skipped.toLocaleString()} unchanged
            </span>
          )}
          {event.errors > 0 && (
            <span className="text-[10px] text-danger">
              {event.errors.toLocaleString()} errors
            </span>
          )}
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="px-4 pb-2.5">
          <p className="text-[10px] text-danger">{error}</p>
        </div>
      )}
    </div>
  );
}
