import { Show, For, Switch, Match, createMemo } from "solid-js";
import { sessionStore } from "../../stores/session";
import InstrumentEditor from "../editors/InstrumentEditor";
import TuningEditor from "../editors/TuningEditor";
import ConstraintsEditor from "../editors/ConstraintsEditor";

export default function Workspace() {
  const activeTab = createMemo(() => {
    const id = sessionStore.activeTabId();
    if (!id) return null;
    return sessionStore.tabs.find((t) => t.id === id) ?? null;
  });

  return (
    <main class="flex-1 flex flex-col overflow-hidden">
      {/* Error banner */}
      <Show when={sessionStore.error()}>
        <div
          class="mx-4 mt-3 mb-1 p-3 rounded text-sm"
          style={{
            background: "rgba(239,68,68,0.1)",
            color: "var(--color-error)",
            border: "1px solid var(--color-error)",
          }}
        >
          {sessionStore.error()}
        </div>
      </Show>

      {/* Tab bar */}
      <Show when={sessionStore.tabs.length > 0}>
        <div
          class="flex items-center gap-0 overflow-x-auto border-b px-1"
          style={{
            background: "var(--color-surface)",
            "border-color": "var(--color-border)",
            "min-height": "32px",
          }}
        >
          <For each={sessionStore.tabs}>
            {(tab) => {
              const isActive = () => sessionStore.activeTabId() === tab.id;
              return (
                <button
                  class="flex items-center gap-1.5 px-3 py-1.5 text-xs transition-colors whitespace-nowrap"
                  style={{
                    background: isActive() ? "var(--color-surface-alt)" : "transparent",
                    color: isActive() ? "var(--color-text)" : "var(--color-text-muted)",
                    "border-bottom": isActive() ? "2px solid var(--color-accent)" : "2px solid transparent",
                  }}
                  onClick={() => sessionStore.setActiveTabId(tab.id)}
                >
                  <span class="text-[10px] opacity-60">
                    {tab.kind === "Instrument" ? "I" : tab.kind === "Tuning" ? "T" : "C"}
                  </span>
                  {tab.title}
                  <span
                    class="ml-1 opacity-40 hover:opacity-100 cursor-pointer"
                    title="Save as XML"
                    onClick={(e) => {
                      e.stopPropagation();
                      sessionStore.saveDocXml(tab.docId);
                    }}
                  >
                    &#x2913;
                  </span>
                  <span
                    class="opacity-40 hover:opacity-100 cursor-pointer"
                    onClick={(e) => {
                      e.stopPropagation();
                      sessionStore.closeTab(tab.id);
                    }}
                  >
                    x
                  </span>
                </button>
              );
            }}
          </For>
        </div>
      </Show>

      {/* Editor area */}
      <div class="flex-1 overflow-auto p-4">
        <Show
          when={activeTab()}
          fallback={
            <Show when={sessionStore.ready()}>
              <div class="flex flex-col items-center justify-center h-full opacity-40">
                <p class="text-lg mb-2">Drop XML files here to get started</p>
                <p class="text-sm">or use the Open File button</p>
                <p class="text-xs mt-4">Double-click a document in the study panel to edit it</p>
              </div>
            </Show>
          }
        >
          {(tab) => (
            <Switch>
              <Match when={tab().kind === "Instrument"}>
                <InstrumentEditor docId={tab().docId} />
              </Match>
              <Match when={tab().kind === "Tuning"}>
                <TuningEditor docId={tab().docId} />
              </Match>
              <Match when={tab().kind === "Constraints"}>
                <ConstraintsEditor docId={tab().docId} />
              </Match>
            </Switch>
          )}
        </Show>
      </div>
    </main>
  );
}
