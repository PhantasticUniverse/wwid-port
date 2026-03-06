import { createSignal, createEffect, Show, For, on, createMemo } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { sessionStore } from "../../stores/session";
import type { ConstraintsData, ConstraintData } from "../../types/documents";
import { formatDisplay } from "../shared/NumberField";

/** Inline constraint bound input with display formatting. */
function BoundInput(props: {
  value: number | undefined;
  onInput: (v: number | undefined) => void;
  title?: string;
}) {
  const [focused, setFocused] = createSignal(false);
  const [local, setLocal] = createSignal(props.value != null ? String(props.value) : "");

  return (
    <input
      type={focused() ? "number" : "text"}
      step="any"
      class="w-full px-1 py-0.5 rounded text-xs text-right tabular-nums"
      style={{
        background: "var(--color-surface-alt)",
        border: "1px solid var(--color-border)",
        color: "var(--color-text)",
      }}
      value={focused() ? local() : (props.value != null ? formatDisplay(props.value, 4) : "")}
      onFocus={() => {
        setLocal(props.value != null ? String(props.value) : "");
        setFocused(true);
      }}
      onInput={(e) => {
        const raw = e.currentTarget.value;
        setLocal(raw);
        const v = parseFloat(raw);
        props.onInput(isNaN(v) ? undefined : v);
      }}
      onBlur={() => setFocused(false)}
      title={props.title}
    />
  );
}

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
                          <BoundInput
                            value={item.c.lowerBound}
                            onInput={(v) => setData("constraint", item.index, "lowerBound", v)}
                            title="Lower optimization bound"
                          />
                        </td>
                        <td class="py-1 px-2">
                          <BoundInput
                            value={item.c.upperBound}
                            onInput={(v) => setData("constraint", item.index, "upperBound", v)}
                            title="Upper optimization bound"
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
