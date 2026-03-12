import { useState, useCallback } from "react";
import { Layout } from "./components/Layout";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { IndexStatus } from "./components/IndexStatus";
import { IndexedLocations } from "./components/IndexedLocations";
import { IndexingPanel } from "./components/IndexingPanel";
import { ModelBanner } from "./components/ModelBanner";
import { Settings } from "./components/Settings";
import { useSearch } from "./hooks/useSearch";
import { useSettings } from "./hooks/useSettings";
import { useKeyboard } from "./hooks/useKeyboard";
import { useModelManager } from "./hooks/useModelManager";
import { useIndexedLocations } from "./hooks/useIndexedLocations";
import { useIndexing } from "./hooks/useIndexing";

type View = "search" | "settings";

function App() {
  const [view, setView] = useState<View>("search");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const { query, results, loading, handleQueryChange, clearSearch } =
    useSearch();
  const settings = useSettings();
  const locations = useIndexedLocations();
  const modelManager = useModelManager();
  const indexing = useIndexing();
  const { selectedIndex, setSelectedIndex, handleKeyDown } = useKeyboard(
    results.length
  );

  const openSettings = useCallback(() => setView("settings"), []);
  const closeSettings = useCallback(() => {
    setView("search");
    settings.refresh();
    locations.refresh();
    modelManager.refresh();
  }, [settings, locations, modelManager]);

  const toggleSidebar = useCallback(
    () => setSidebarCollapsed((prev) => !prev),
    []
  );

  if (view === "settings") {
    return (
      <Layout>
        <Settings
          roots={locations.roots}
          stats={settings.stats}
          loading={settings.loading}
          error={settings.error}
          onReindex={indexing.startIndexing}
          onClose={closeSettings}
          modelInfo={modelManager.model}
          modelStatus={modelManager.status}
          modelProgress={modelManager.progress}
          modelError={modelManager.error}
          modelLoading={modelManager.loading}
          onDownloadModel={modelManager.startDownload}
          allModels={modelManager.allModels}
          downloadingModelId={modelManager.downloadingModelId}
          onDownloadModelById={modelManager.startDownloadById}
          indexing={indexing.indexing}
          indexingEvent={indexing.event}
          indexingMode={indexing.mode}
          indexingError={indexing.error}
          onSetIndexingMode={indexing.setMode}
          onCancelIndexing={indexing.cancelIndexing}
        />
      </Layout>
    );
  }

  return (
    <Layout>
      <div className="flex flex-1 min-h-0">
        <IndexedLocations
          roots={locations.roots}
          collapsed={sidebarCollapsed}
          onToggleCollapsed={toggleSidebar}
          onToggleRoot={locations.toggleRoot}
          onAddFolder={locations.handleAddFolder}
          onRemoveFolder={locations.handleRemoveFolder}
          toggling={locations.toggling}
          loading={locations.loading}
        />
        <div className="flex flex-col flex-1 min-w-0">
          <ModelBanner
            status={modelManager.status}
            progress={modelManager.progress}
            onDownload={modelManager.startDownload}
          />
          <SearchBar
            query={query}
            onChange={handleQueryChange}
            onClear={clearSearch}
            onKeyDown={handleKeyDown}
            onSettingsClick={openSettings}
            onToggleSidebar={toggleSidebar}
            sidebarCollapsed={sidebarCollapsed}
            loading={loading}
          />
          <IndexingPanel
            indexing={indexing.indexing}
            event={indexing.event}
            mode={indexing.mode}
            error={indexing.error}
            stats={settings.stats}
            onStartIndexing={() => {
              indexing.startIndexing();
              setTimeout(() => settings.refresh(), 2000);
            }}
            onCancelIndexing={indexing.cancelIndexing}
            onSetMode={indexing.setMode}
          />
          <ResultList
            results={results}
            selectedIndex={selectedIndex}
            onSelect={setSelectedIndex}
            loading={loading}
            query={query}
            modelStatus={modelManager.status}
            hasIndexedFiles={(settings.stats?.total_files ?? 0) > 0}
          />
          <IndexStatus stats={settings.stats} />
        </div>
      </div>
    </Layout>
  );
}

export default App;
