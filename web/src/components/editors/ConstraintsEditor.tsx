import { createSignal, createEffect, Show, For, on, createMemo } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { sessionStore } from "../../stores/session";
import type { ConstraintsData, ConstraintData } from "../../types/documents";

const EMPTY_CONSTRAINTS: ConstraintsData = {
  constraintsName: "",
  objectiveDisplayName: "",
  objectiveFunctionName: "",
  numberOfHoles: 0,
  constraint: [],
};

export default function ConstraintsEditor(props: { docId: number }) {
  const [data, setData] = createStore<ConstraintsData>({ ...EMPTY_CONSTRAINTS });
  const [loaded, setLoaded] = createSignal(false);

  createEffect(
    on(
      () => props.docId,
      async (docId) => {
        setLoaded(false);
        const constraints = await sessionStore.getConstraints(docId);
        setData(reconcile(constraints));
        setLoaded(true);
      }
    )
  );

  async function sync() {
    await sessionStore.setConstraints(props.docId, structuredClone(data) as ConstraintsData);
  }

  const groups = createMemo(() => {
    if (!data.constraint) return [];
    const map = new Map<string, { category: string; items: { index: number; c: ConstraintData }[] }>();
    data.constraint.forEach((c, i) => {
      let group = map.get(c.category);
      if (!group) {
        group = { category: c.category, items: [] };
        map.set(c.category, group);
      }
      group.items.push({ index: i, c });
    });
    return [...map.values()];
  });

  return (
    <Show when={loaded()} fallback={<p class="text-sm opacity-50">Loading...</p>}>
      <div class="flex flex-col gap-5 max-w-3xl">
        {/* Header */}
        <section class="flex flex-col gap-2">
          <div class="flex items-center gap-3">
            <label class="text-xs w-32" style={{ color: "var(--color-text-muted)" }}>
              Name
            </label>
            <input
              class="flex-1 px-2 py-1 rounded text-sm"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={data.constraintsName}
              onInput={(e) => setData("constraintsName", e.currentTarget.value)}
              onBlur={sync}
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs w-32" style={{ color: "var(--color-text-muted)" }}>
              Objective
            </label>
            <span class="text-sm">{data.objectiveDisplayName}</span>
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs w-32" style={{ color: "var(--color-text-muted)" }}>
              # Holes
            </label>
            <span class="text-sm tabular-nums">{data.numberOfHoles}</span>
          </div>
        </section>

        {/* Category-grouped constraint tables */}
        <For each={groups()}>
          {(group) => (
            <section>
              <h3
                class="text-xs font-semibold uppercase tracking-wider mb-2"
                style={{ color: "var(--color-text-muted)" }}
              >
                {group.category}
              </h3>
              <table class="w-full text-xs border-collapse">
                <thead>
                  <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                    <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                      Constraint
                    </th>
                    <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                      Type
                    </th>
                    <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                      Lower Bound
                    </th>
                    <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                      Upper Bound
                    </th>
                  </tr>
                </thead>
                <tbody onFocusOut={sync}>
                  <For each={group.items}>
                    {(item) => (
                      <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                        <td class="py-1 px-2">{item.c.displayName}</td>
                        <td
                          class="py-1 px-2"
                          style={{ color: "var(--color-text-muted)" }}
                        >
                          {item.c.type}
                        </td>
                        <td class="py-1 px-2">
                          <input
                            type="number"
                            step="any"
                            class="w-full px-1 py-0.5 rounded text-xs text-right tabular-nums"
                            style={{
                              background: "var(--color-surface-alt)",
                              border: "1px solid var(--color-border)",
                              color: "var(--color-text)",
                            }}
                            value={item.c.lowerBound ?? ""}
                            onInput={(e) => {
                              const v = parseFloat(e.currentTarget.value);
                              setData(
                                "constraint",
                                item.index,
                                "lowerBound",
                                isNaN(v) ? undefined : v
                              );
                            }}
                          />
                        </td>
                        <td class="py-1 px-2">
                          <input
                            type="number"
                            step="any"
                            class="w-full px-1 py-0.5 rounded text-xs text-right tabular-nums"
                            style={{
                              background: "var(--color-surface-alt)",
                              border: "1px solid var(--color-border)",
                              color: "var(--color-text)",
                            }}
                            value={item.c.upperBound ?? ""}
                            onInput={(e) => {
                              const v = parseFloat(e.currentTarget.value);
                              setData(
                                "constraint",
                                item.index,
                                "upperBound",
                                isNaN(v) ? undefined : v
                              );
                            }}
                          />
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </section>
          )}
        </For>
      </div>
    </Show>
  );
}
