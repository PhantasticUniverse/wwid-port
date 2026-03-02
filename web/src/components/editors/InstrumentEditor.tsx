import { createSignal, createEffect, Show, For, on } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { sessionStore } from "../../stores/session";
import type { InstrumentData, LengthType } from "../../types/documents";
import NumberField from "../shared/NumberField";

const EMPTY_INST: InstrumentData = {
  name: "",
  lengthType: "in",
  mouthpiece: { position: 0 },
  borePoint: [],
  hole: [],
  termination: { flangeDiameter: 0 },
};

export default function InstrumentEditor(props: { docId: number }) {
  const [data, setData] = createStore<InstrumentData>({ ...EMPTY_INST });
  const [loaded, setLoaded] = createSignal(false);

  createEffect(
    on(
      () => [props.docId, sessionStore.calibrationCount()] as const,
      async ([docId]) => {
        setLoaded(false);
        const inst = await sessionStore.getInstrument(docId);
        setData(reconcile(inst));
        setLoaded(true);
      }
    )
  );

  async function sync() {
    await sessionStore.setInstrument(props.docId, structuredClone(data) as InstrumentData);
  }

  return (
    <Show when={loaded()} fallback={<p class="text-sm opacity-50">Loading...</p>}>
      <div class="flex flex-col gap-5 max-w-3xl">
        {/* Header */}
        <section class="flex flex-col gap-2">
          <div class="flex items-center gap-3">
            <label class="text-xs w-20" style={{ color: "var(--color-text-muted)" }}>
              Name
            </label>
            <input
              class="flex-1 px-2 py-1 rounded text-sm"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={data.name}
              onInput={(e) => setData("name", e.currentTarget.value)}
              onBlur={sync}
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs w-20" style={{ color: "var(--color-text-muted)" }}>
              Description
            </label>
            <input
              class="flex-1 px-2 py-1 rounded text-sm"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={data.description ?? ""}
              onInput={(e) =>
                setData("description", e.currentTarget.value || undefined)
              }
              onBlur={sync}
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs w-20" style={{ color: "var(--color-text-muted)" }}>
              Length Type
            </label>
            <select
              class="px-2 py-1 rounded text-sm"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={data.lengthType}
              onChange={(e) => {
                setData("lengthType", e.currentTarget.value as LengthType);
                sync();
              }}
            >
              <option value="in">in</option>
              <option value="mm">mm</option>
              <option value="cm">cm</option>
              <option value="m">m</option>
              <option value="ft">ft</option>
            </select>
          </div>
        </section>

        {/* Mouthpiece */}
        <section onFocusOut={sync}>
          <h3
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Mouthpiece
          </h3>
          <div class="grid grid-cols-2 gap-2">
            <NumberField
              label="Position"
              value={data.mouthpiece.position}
              onChange={(v) => {
                if (v != null) setData("mouthpiece", "position", v);
              }}
              step={0.001}
            />
            <Show when={data.mouthpiece.fipple}>
              <NumberField
                label="Window Length"
                value={data.mouthpiece.fipple!.windowLength}
                onChange={(v) => {
                  if (v != null) setData("mouthpiece", "fipple", "windowLength", v);
                }}
                step={0.001}
              />
              <NumberField
                label="Window Width"
                value={data.mouthpiece.fipple!.windowWidth}
                onChange={(v) => {
                  if (v != null) setData("mouthpiece", "fipple", "windowWidth", v);
                }}
                step={0.001}
              />
              <NumberField
                label="Fipple Factor"
                value={data.mouthpiece.fipple!.fippleFactor}
                onChange={(v) => setData("mouthpiece", "fipple", "fippleFactor", v)}
                step={0.01}
                nullable
              />
              <NumberField
                label="Window Height"
                value={data.mouthpiece.fipple!.windowHeight}
                onChange={(v) => setData("mouthpiece", "fipple", "windowHeight", v)}
                step={0.001}
                nullable
              />
              <NumberField
                label="Windway Length"
                value={data.mouthpiece.fipple!.windwayLength}
                onChange={(v) => setData("mouthpiece", "fipple", "windwayLength", v)}
                step={0.001}
                nullable
              />
              <NumberField
                label="Windway Height"
                value={data.mouthpiece.fipple!.windwayHeight}
                onChange={(v) => setData("mouthpiece", "fipple", "windwayHeight", v)}
                step={0.001}
                nullable
              />
            </Show>
          </div>
        </section>

        {/* Bore profile */}
        <section>
          <h3
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Bore Profile
          </h3>
          <table class="w-full text-xs border-collapse">
            <thead>
              <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                  Name
                </th>
                <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                  Position
                </th>
                <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                  Diameter
                </th>
              </tr>
            </thead>
            <tbody onFocusOut={sync}>
              <For each={data.borePoint}>
                {(bp, i) => (
                  <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                    <td class="py-1 px-2">
                      <input
                        class="w-full px-1 py-0.5 rounded text-xs"
                        style={{
                          background: "var(--color-surface-alt)",
                          border: "1px solid var(--color-border)",
                          color: "var(--color-text)",
                        }}
                        value={bp.name ?? ""}
                        onInput={(e) =>
                          setData(
                            "borePoint",
                            i(),
                            "name",
                            e.currentTarget.value || undefined
                          )
                        }
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
                        value={bp.borePosition}
                        onInput={(e) => {
                          const v = parseFloat(e.currentTarget.value);
                          if (!isNaN(v)) setData("borePoint", i(), "borePosition", v);
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
                        value={bp.boreDiameter}
                        onInput={(e) => {
                          const v = parseFloat(e.currentTarget.value);
                          if (!isNaN(v)) setData("borePoint", i(), "boreDiameter", v);
                        }}
                      />
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </section>

        {/* Holes */}
        <section>
          <h3
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Holes
          </h3>
          <Show
            when={data.hole.length > 0}
            fallback={
              <p class="text-xs" style={{ color: "var(--color-text-muted)" }}>
                No holes
              </p>
            }
          >
            <table class="w-full text-xs border-collapse">
              <thead>
                <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                  <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    #
                  </th>
                  <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Name
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Position
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Spacing
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Diameter
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Height
                  </th>
                </tr>
              </thead>
              <tbody onFocusOut={sync}>
                <For each={data.hole}>
                  {(hole, i) => {
                    const spacing = () => {
                      if (i() === 0) return "—";
                      const prev = data.hole[i() - 1].borePosition;
                      return (hole.borePosition - prev).toFixed(4);
                    };
                    return (
                      <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                        <td class="py-1 px-2 tabular-nums" style={{ color: "var(--color-text-muted)" }}>
                          {i() + 1}
                        </td>
                        <td class="py-1 px-2">
                          <input
                            class="w-full px-1 py-0.5 rounded text-xs"
                            style={{
                              background: "var(--color-surface-alt)",
                              border: "1px solid var(--color-border)",
                              color: "var(--color-text)",
                            }}
                            value={hole.name ?? ""}
                            onInput={(e) =>
                              setData(
                                "hole",
                                i(),
                                "name",
                                e.currentTarget.value || undefined
                              )
                            }
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
                            value={hole.borePosition}
                            onInput={(e) => {
                              const v = parseFloat(e.currentTarget.value);
                              if (!isNaN(v)) setData("hole", i(), "borePosition", v);
                            }}
                          />
                        </td>
                        <td
                          class="py-1 px-2 text-right tabular-nums"
                          style={{ color: "var(--color-text-muted)" }}
                        >
                          {spacing()}
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
                            value={hole.diameter}
                            onInput={(e) => {
                              const v = parseFloat(e.currentTarget.value);
                              if (!isNaN(v)) setData("hole", i(), "diameter", v);
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
                            value={hole.height}
                            onInput={(e) => {
                              const v = parseFloat(e.currentTarget.value);
                              if (!isNaN(v)) setData("hole", i(), "height", v);
                            }}
                          />
                        </td>
                      </tr>
                    );
                  }}
                </For>
              </tbody>
            </table>
          </Show>
        </section>

        {/* Termination */}
        <section>
          <h3
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Termination
          </h3>
          <div onFocusOut={sync}>
            <NumberField
              label="Flange Diameter"
              value={data.termination.flangeDiameter}
              onChange={(v) => {
                if (v != null) setData("termination", "flangeDiameter", v);
              }}
              step={0.001}
            />
          </div>
        </section>
      </div>
    </Show>
  );
}
