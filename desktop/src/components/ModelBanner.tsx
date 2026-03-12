import type { ModelStatusValue, DownloadProgress } from "../lib/types";

interface ModelBannerProps {
  status: ModelStatusValue;
  progress: DownloadProgress | null;
  onDownload: () => void;
}

export function ModelBanner({ status, progress, onDownload }: ModelBannerProps) {
  if (status === "installed") return null;

  if (status === "downloading") {
    const percent = progress?.percent ?? null;
    const label = progress
      ? `Downloading ${progress.file_index + 1}/${progress.file_count}…`
      : "Preparing download…";

    return (
      <div className="flex items-center gap-3 px-4 py-2 bg-accent/8 border-b border-accent/15">
        <div className="w-3.5 h-3.5 border-2 border-accent border-t-transparent rounded-full animate-spin shrink-0" />
        <span className="text-xs text-text-secondary">{label}</span>
        {percent !== null && (
          <div className="flex-1 max-w-48 h-1.5 bg-bg-tertiary rounded-full overflow-hidden">
            <div
              className="h-full bg-accent rounded-full transition-all duration-300"
              style={{ width: `${percent}%` }}
            />
          </div>
        )}
        {percent !== null && (
          <span className="text-[10px] text-text-muted tabular-nums">
            {Math.round(percent)}%
          </span>
        )}
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 px-4 py-2 bg-warning/8 border-b border-warning/15">
      <svg
        className="w-4 h-4 text-warning shrink-0"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth={2}
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 00-2.455 2.456z"
        />
      </svg>
      <span className="text-xs text-text-secondary">
        Semantic search requires a local AI model.
      </span>
      <button
        onClick={onDownload}
        className="ml-auto text-xs font-medium px-3 py-1 bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors"
      >
        Download AI Model
      </button>
    </div>
  );
}
