import { useRef, useEffect } from "react";
import type { SearchResult, ModelStatusValue } from "../lib/types";
import { ResultItem } from "./ResultItem";
import { openFile, openFileAtLine } from "../lib/tauri";

interface ResultListProps {
  results: SearchResult[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  loading: boolean;
  query: string;
  modelStatus: ModelStatusValue;
  hasIndexedFiles: boolean;
}

export function ResultList({
  results,
  selectedIndex,
  onSelect,
  loading,
  query,
  modelStatus,
  hasIndexedFiles,
}: ResultListProps) {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (selectedIndex >= 0 && listRef.current) {
      const items = listRef.current.querySelectorAll("[data-result-item]");
      items[selectedIndex]?.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  useEffect(() => {
    const handleEnter = (e: KeyboardEvent) => {
      if (e.key === "Enter" && selectedIndex >= 0 && results[selectedIndex]) {
        const r = results[selectedIndex];
        if (e.metaKey || e.ctrlKey) {
          import("../lib/tauri").then((t) => t.revealInFolder(r.path));
        } else if (r.line_start != null) {
          openFileAtLine(r.path, r.line_start);
        } else {
          openFile(r.path);
        }
      }
    };
    window.addEventListener("keydown", handleEnter);
    return () => window.removeEventListener("keydown", handleEnter);
  }, [selectedIndex, results]);

  if (!query) {
    const modelDone = modelStatus === "installed";
    const indexDone = hasIndexedFiles;

    return (
      <div className="flex-1 flex flex-col items-center justify-center text-text-muted px-8">
        <p className="text-lg font-medium text-text-primary mb-5">
          Get started
        </p>
        <div className="flex flex-col gap-3 w-full max-w-xs">
          <StepRow
            num={1}
            done={modelDone}
            label="Download the local AI model"
            hint="Enables semantic search by meaning"
          />
          <StepRow
            num={2}
            done={indexDone}
            label="Index your files"
            hint="Scan folders to make them searchable"
          />
          <StepRow
            num={3}
            done={modelDone && indexDone}
            label="Search by name, content, or context"
            hint="All processing happens locally"
          />
        </div>
        <div className="flex gap-3 mt-8 text-xs">
          <kbd className="px-2 py-1 bg-bg-tertiary rounded border border-border text-text-secondary">
            /
          </kbd>
          <span className="text-text-muted self-center">to focus search</span>
          <kbd className="px-2 py-1 bg-bg-tertiary rounded border border-border text-text-secondary ml-2">
            Arrow keys
          </kbd>
          <span className="text-text-muted self-center">to navigate</span>
          <kbd className="px-2 py-1 bg-bg-tertiary rounded border border-border text-text-secondary ml-2">
            Enter
          </kbd>
          <span className="text-text-muted self-center">to open</span>
        </div>
      </div>
    );
  }

  if (loading && results.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (results.length === 0 && query) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-text-muted">
        <p className="text-lg font-medium">No results found</p>
        <p className="text-sm mt-1">
          Try different keywords or add folders in Settings
        </p>
      </div>
    );
  }

  return (
    <div ref={listRef} className="flex-1 overflow-y-auto">
      <div className="px-5 py-2 text-xs text-text-muted border-b border-border/30">
        {results.length} result{results.length !== 1 ? "s" : ""}
      </div>
      {results.map((result, index) => (
        <div key={`${result.path}-${index}`} data-result-item>
          <ResultItem
            result={result}
            selected={index === selectedIndex}
            onSelect={() => onSelect(index)}
            query={query}
          />
        </div>
      ))}
    </div>
  );
}

function StepRow({
  num,
  done,
  label,
  hint,
}: {
  num: number;
  done: boolean;
  label: string;
  hint: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <div
        className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 text-xs font-semibold ${
          done
            ? "bg-success/20 text-success"
            : "bg-bg-tertiary text-text-muted border border-border"
        }`}
      >
        {done ? (
          <svg
            className="w-3.5 h-3.5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={3}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M4.5 12.75l6 6 9-13.5"
            />
          </svg>
        ) : (
          num
        )}
      </div>
      <div className="min-w-0">
        <p
          className={`text-sm leading-tight ${
            done ? "text-text-secondary" : "text-text-primary"
          }`}
        >
          {label}
        </p>
        <p className="text-[11px] text-text-muted leading-tight mt-0.5">
          {hint}
        </p>
      </div>
    </div>
  );
}
