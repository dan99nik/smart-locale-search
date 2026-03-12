import type {
  IndexedRoot,
  IndexStats,
  IndexingEvent,
  IndexingMode,
  ModelInfo,
  DownloadProgress,
  ModelStatusValue,
} from "../lib/types";
import { ModelManagement } from "./ModelManagement";
import { IndexingPanel } from "./IndexingPanel";

interface SettingsProps {
  roots: IndexedRoot[];
  stats: IndexStats | null;
  loading: boolean;
  error: string | null;
  onReindex: () => void;
  onClose: () => void;
  modelInfo: ModelInfo | null;
  modelStatus: ModelStatusValue;
  modelProgress: DownloadProgress | null;
  modelError: string | null;
  modelLoading: boolean;
  onDownloadModel: () => void;
  allModels?: ModelInfo[];
  downloadingModelId?: string | null;
  onDownloadModelById?: (modelId: string) => void;
  indexing: boolean;
  indexingEvent: IndexingEvent | null;
  indexingMode: IndexingMode;
  indexingError: string | null;
  onSetIndexingMode: (mode: IndexingMode) => void;
  onCancelIndexing: () => void;
}

export function Settings({
  roots,
  stats,
  loading: _loading,
  error,
  onReindex,
  onClose,
  modelInfo,
  modelStatus,
  modelProgress,
  modelError,
  modelLoading,
  onDownloadModel,
  allModels,
  downloadingModelId,
  onDownloadModelById,
  indexing,
  indexingEvent,
  indexingMode,
  indexingError,
  onSetIndexingMode,
  onCancelIndexing,
}: SettingsProps) {
  const enabledCount = roots.filter((r) => r.enabled).length;

  return (
    <div className="flex flex-col h-full bg-bg-primary">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-border bg-bg-secondary">
        <h1 className="text-lg font-semibold text-text-primary">Settings</h1>
        <button
          onClick={onClose}
          className="p-1.5 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-5 space-y-6">
        {/* Privacy Notice */}
        <div className="bg-accent/10 border border-accent/20 rounded-xl p-4">
          <div className="flex items-center gap-2 mb-2">
            <svg
              className="w-5 h-5 text-accent"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z"
              />
            </svg>
            <span className="font-medium text-accent">Privacy First</span>
          </div>
          <p className="text-sm text-text-secondary leading-relaxed">
            All indexing and search happens locally. Your files never leave your
            computer. No cloud APIs, no accounts, no tracking. Smart Local
            Search works fully offline.
          </p>
        </div>

        {/* Indexed Locations Summary (read-only, management is on main screen) */}
        <section>
          <h2 className="text-sm font-semibold text-text-primary mb-3">
            Indexed Locations
          </h2>
          <p className="text-xs text-text-muted mb-3">
            Manage indexed locations from the sidebar on the main screen.
          </p>
          {roots.length === 0 ? (
            <div className="text-sm text-text-muted py-4 text-center border border-dashed border-border rounded-xl">
              No folders configured yet.
            </div>
          ) : (
            <div className="space-y-1.5">
              {roots.map((root) => (
                <div
                  key={root.id}
                  className={`flex items-center gap-2 px-3 py-2 bg-bg-secondary rounded-lg border border-border ${
                    !root.enabled ? "opacity-50" : ""
                  }`}
                >
                  <svg
                    className="w-4 h-4 text-text-muted shrink-0"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={1.5}
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z"
                    />
                  </svg>
                  <span className="text-sm text-text-primary truncate flex-1">
                    {root.path}
                  </span>
                  <span
                    className={`text-[10px] px-1.5 py-0.5 rounded-full ${
                      root.enabled
                        ? "bg-success/15 text-success"
                        : "bg-bg-tertiary text-text-muted"
                    }`}
                  >
                    {root.enabled ? "Active" : "Excluded"}
                  </span>
                </div>
              ))}
              <p className="text-[11px] text-text-muted mt-2">
                {enabledCount} of {roots.length} location
                {roots.length !== 1 ? "s" : ""} active
              </p>
            </div>
          )}
        </section>

        {/* Index Statistics */}
        {stats && (
          <section>
            <h2 className="text-sm font-semibold text-text-primary mb-3">
              Index Statistics
            </h2>
            <div className="grid grid-cols-2 gap-3">
              <StatCard
                label="Files"
                value={stats.total_files.toLocaleString()}
              />
              <StatCard
                label="Text Chunks"
                value={stats.total_chunks.toLocaleString()}
              />
              <StatCard
                label="Vectors"
                value={stats.total_vectors.toLocaleString()}
              />
              <StatCard
                label="Folders"
                value={stats.total_roots.toString()}
              />
            </div>
          </section>
        )}

        {/* AI Models */}
        <ModelManagement
          model={modelInfo}
          status={modelStatus}
          progress={modelProgress}
          error={modelError}
          loading={modelLoading}
          onDownload={onDownloadModel}
          allModels={allModels}
          downloadingModelId={downloadingModelId}
          onDownloadById={onDownloadModelById}
        />

        {/* Indexing */}
        <section>
          <h2 className="text-sm font-semibold text-text-primary mb-3">
            Indexing
          </h2>
          <IndexingPanel
            indexing={indexing}
            event={indexingEvent}
            mode={indexingMode}
            error={indexingError}
            stats={stats}
            onStartIndexing={onReindex}
            onCancelIndexing={onCancelIndexing}
            onSetMode={onSetIndexingMode}
            embedded
          />
        </section>

        {error && (
          <div className="text-sm text-danger bg-danger/10 p-3 rounded-xl">
            {error}
          </div>
        )}

        {/* About */}
        <section className="pt-4 border-t border-border">
          <h2 className="text-sm font-semibold text-text-primary mb-2">
            About
          </h2>
          <p className="text-sm text-text-secondary">
            Smart Local Search v0.1.0
          </p>
          <p className="text-xs text-text-muted mt-1">
            Privacy-first local AI search engine. Open source under MIT License.
          </p>
        </section>
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-bg-secondary border border-border rounded-xl p-3">
      <p className="text-xs text-text-muted">{label}</p>
      <p className="text-lg font-semibold text-text-primary">{value}</p>
    </div>
  );
}
