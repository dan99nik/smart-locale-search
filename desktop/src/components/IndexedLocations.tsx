import { useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { IndexedRoot } from "../lib/types";

interface IndexedLocationsProps {
  roots: IndexedRoot[];
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onToggleRoot: (id: number, enabled: boolean) => void;
  onAddFolder: (path: string) => void;
  onRemoveFolder: (id: number) => void;
  toggling: number | null;
  loading: boolean;
}

function shortenPath(path: string): string {
  const home = path.replace(/^\/Users\/[^/]+/, "~");
  if (home !== path) return home;
  const win = path.replace(/^C:\\Users\\[^\\]+/, "~");
  if (win !== path) return win;
  return path;
}

function folderName(path: string): string {
  const parts = path.replace(/[/\\]+$/, "").split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

export function IndexedLocations({
  roots,
  collapsed,
  onToggleCollapsed,
  onToggleRoot,
  onAddFolder,
  onRemoveFolder,
  toggling,
  loading,
}: IndexedLocationsProps) {
  const handlePickFolder = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      onAddFolder(selected as string);
    }
  }, [onAddFolder]);

  const enabledCount = roots.filter((r) => r.enabled).length;

  if (collapsed) {
    return (
      <button
        onClick={onToggleCollapsed}
        className="flex items-center gap-1.5 px-2 py-1.5 text-xs text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors"
        title="Show indexed locations"
      >
        <svg
          className="w-4 h-4"
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
        <span>
          {enabledCount}/{roots.length}
        </span>
        <svg
          className="w-3 h-3"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth={2}
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M8.25 4.5l7.5 7.5-7.5 7.5"
          />
        </svg>
      </button>
    );
  }

  return (
    <div className="w-64 shrink-0 border-r border-border bg-bg-secondary flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2.5 border-b border-border/50">
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider">
          Indexed Locations
        </h2>
        <div className="flex items-center gap-1">
          <button
            onClick={handlePickFolder}
            disabled={loading}
            className="p-1 text-text-muted hover:text-accent hover:bg-accent/10 rounded transition-colors disabled:opacity-50"
            title="Add folder"
          >
            <svg
              className="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M12 4.5v15m7.5-7.5h-15"
              />
            </svg>
          </button>
          <button
            onClick={onToggleCollapsed}
            className="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
            title="Collapse panel"
          >
            <svg
              className="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M15.75 19.5L8.25 12l7.5-7.5"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* Folder list */}
      <div className="flex-1 overflow-y-auto py-1">
        {roots.length === 0 && !loading && (
          <div className="px-3 py-6 text-center text-xs text-text-muted">
            No folders configured.
            <br />
            Click + to add one.
          </div>
        )}
        {loading && roots.length === 0 && (
          <div className="px-3 py-6 flex items-center justify-center">
            <div className="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
          </div>
        )}
        {roots.map((root) => (
          <RootItem
            key={root.id}
            root={root}
            toggling={toggling === root.id}
            onToggle={(enabled) => onToggleRoot(root.id, enabled)}
            onRemove={() => onRemoveFolder(root.id)}
          />
        ))}
      </div>

      {/* Footer summary */}
      {roots.length > 0 && (
        <div className="px-3 py-2 border-t border-border/50 text-[10px] text-text-muted">
          {enabledCount} of {roots.length} location
          {roots.length !== 1 ? "s" : ""} active
        </div>
      )}
    </div>
  );
}

function RootItem({
  root,
  toggling,
  onToggle,
  onRemove,
}: {
  root: IndexedRoot;
  toggling: boolean;
  onToggle: (enabled: boolean) => void;
  onRemove: () => void;
}) {
  return (
    <div
      className={`group flex items-center gap-2 px-3 py-1.5 hover:bg-bg-hover/50 transition-colors ${
        !root.enabled ? "opacity-50" : ""
      }`}
    >
      <label className="flex items-center gap-2 flex-1 min-w-0 cursor-pointer">
        <input
          type="checkbox"
          checked={root.enabled}
          disabled={toggling}
          onChange={(e) => onToggle(e.target.checked)}
          className="w-3.5 h-3.5 rounded border-border text-accent focus:ring-accent/30 focus:ring-offset-0 bg-bg-primary shrink-0 cursor-pointer"
        />
        <div className="min-w-0 flex-1">
          <p className="text-[13px] text-text-primary truncate leading-tight">
            {folderName(root.path)}
          </p>
          <p className="text-[10px] text-text-muted truncate leading-tight">
            {shortenPath(root.path)}
          </p>
        </div>
      </label>
      <button
        onClick={onRemove}
        className="p-0.5 text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-all shrink-0"
        title="Remove from index"
      >
        <svg
          className="w-3.5 h-3.5"
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
  );
}
