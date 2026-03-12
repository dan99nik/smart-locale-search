import type { ModelInfo, DownloadProgress, ModelStatusValue } from "../lib/types";

interface ModelManagementProps {
  model: ModelInfo | null;
  status: ModelStatusValue;
  progress: DownloadProgress | null;
  error: string | null;
  loading: boolean;
  onDownload: () => void;
  allModels?: ModelInfo[];
  downloadingModelId?: string | null;
  onDownloadById?: (modelId: string) => void;
}

const MODEL_DESCRIPTIONS: Record<string, string> = {
  "multilingual-e5-small": "384-dim embeddings for semantic text search",
  "open-clip-vit-b-32": "512-dim CLIP embeddings for visual image search",
};

const MODEL_SIZE_HINTS: Record<string, string> = {
  "multilingual-e5-small": "~134 MB",
  "open-clip-vit-b-32": "~303 MB",
};

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function statusLabel(status: ModelStatusValue): { text: string; color: string } {
  switch (status) {
    case "installed":
      return { text: "Installed", color: "text-success" };
    case "downloading":
      return { text: "Downloading...", color: "text-accent" };
    case "failed":
      return { text: "Failed", color: "text-danger" };
    default:
      return { text: "Not installed", color: "text-warning" };
  }
}

function overallPercent(progress: DownloadProgress): number | null {
  if (progress.percent == null) return null;
  const fileWeight = 100 / progress.file_count;
  return progress.file_index * fileWeight + (progress.percent / 100) * fileWeight;
}

function ModelCard({
  model,
  isDownloading,
  progress,
  loading,
  onDownload,
}: {
  model: ModelInfo;
  isDownloading: boolean;
  progress: DownloadProgress | null;
  loading: boolean;
  onDownload: () => void;
}) {
  const effectiveStatus: ModelStatusValue = isDownloading ? "downloading" : model.status;
  const sl = statusLabel(effectiveStatus);
  const pct = isDownloading && progress ? overallPercent(progress) : null;
  const desc = MODEL_DESCRIPTIONS[model.id] ?? "AI model";
  const sizeHint = MODEL_SIZE_HINTS[model.id];

  return (
    <div className="bg-bg-secondary border border-border rounded-xl p-4 space-y-3">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-text-primary">
            {model.display_name}
          </p>
          <p className="text-xs text-text-muted mt-0.5">{desc}</p>
        </div>
        <span className={`text-xs font-medium ${sl.color}`}>{sl.text}</span>
      </div>

      {model.model_path && (
        <p className="text-[11px] text-text-muted truncate" title={model.model_path}>
          {shortenModelPath(model.model_path)}
        </p>
      )}

      {effectiveStatus === "installed" && (
        <div className="flex flex-wrap gap-3">
          {model.files.map((f) => (
            <div
              key={f.filename}
              className="flex items-center gap-1.5 text-[11px] text-text-muted"
            >
              <svg className="w-3 h-3 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 12.75l6 6 9-13.5" />
              </svg>
              <span>{f.filename}</span>
              {f.actual_size != null && (
                <span className="text-text-muted/60">
                  ({formatBytes(f.actual_size)})
                </span>
              )}
            </div>
          ))}
        </div>
      )}

      {isDownloading && (
        <div className="space-y-1.5">
          <div className="w-full h-2 bg-bg-primary rounded-full overflow-hidden">
            <div
              className="h-full bg-accent rounded-full transition-all duration-300 ease-out"
              style={{ width: `${pct ?? 0}%` }}
            />
          </div>
          <div className="flex justify-between text-[11px] text-text-muted">
            <span>
              {progress
                ? `Downloading ${progress.file} (${progress.file_index + 1}/${progress.file_count})`
                : "Starting download..."}
            </span>
            <span>
              {progress?.downloaded_bytes != null
                ? formatBytes(progress.downloaded_bytes)
                : ""}
              {pct != null ? ` \u2022 ${Math.round(pct)}%` : ""}
            </span>
          </div>
        </div>
      )}

      {effectiveStatus !== "installed" && !isDownloading && (
        <button
          onClick={onDownload}
          disabled={loading}
          className="w-full py-2 text-sm font-medium rounded-lg transition-colors disabled:opacity-50
            bg-accent hover:bg-accent-hover text-white disabled:hover:bg-accent"
        >
          {effectiveStatus === "failed" ? "Retry Download" : "Download Model"}
        </button>
      )}

      {effectiveStatus === "not_installed" && !isDownloading && sizeHint && (
        <p className="text-[11px] text-text-muted leading-relaxed">
          {sizeHint} — downloaded once from Hugging Face. No user data is sent.
        </p>
      )}
    </div>
  );
}

export function ModelManagement({
  model,
  status: _status,
  progress,
  error,
  loading,
  onDownload,
  allModels,
  downloadingModelId,
  onDownloadById,
}: ModelManagementProps) {
  const models = allModels && allModels.length > 0 ? allModels : model ? [model] : [];

  return (
    <section>
      <h2 className="text-sm font-semibold text-text-primary mb-3">
        AI Models
      </h2>
      <div className="space-y-3">
        {models.map((m) => (
          <ModelCard
            key={m.id}
            model={m}
            isDownloading={downloadingModelId === m.id}
            progress={downloadingModelId === m.id ? progress : null}
            loading={loading}
            onDownload={() => {
              if (onDownloadById) {
                onDownloadById(m.id);
              } else {
                onDownload();
              }
            }}
          />
        ))}
        {models.length === 0 && (
          <p className="text-sm text-text-muted">Loading model information...</p>
        )}
      </div>

      {error && (
        <p className="text-xs text-danger bg-danger/10 rounded-lg px-3 py-2 mt-3">
          {error}
        </p>
      )}
    </section>
  );
}

function shortenModelPath(path: string): string {
  return path.replace(/^\/Users\/[^/]+/, "~");
}
