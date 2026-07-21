import { createSignal, createEffect, Show, For, on } from "solid-js";
import { createStore, reconcile, produce } from "solid-js/store";
import { sessionStore } from "../../stores/session";
import type { InstrumentData, LengthType } from "../../types/documents";
import NumberField, { formatDisplay } from "../shared/NumberField";
import HelpLink from "../reference/HelpLink";

/** Inline table number input with display formatting (shows 4 sig digits when not focused). */
function InlineNum(props: {
  value: number;
  onInput: (v: number) => void;
  title?: string;
}) {
  const [focused, setFocused] = createSignal(false);
  const [local, setLocal] = createSignal(String(props.value));

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
      value={focused() ? local() : formatDisplay(props.value, 4)}
      onFocus={() => {
        setLocal(String(props.value));
        setFocused(true);
      }}
      onInput={(e) => {
        const raw = e.currentTarget.value;
        setLocal(raw);
        const v = parseFloat(raw);
        if (!isNaN(v)) props.onInput(v);
      }}
      onBlur={() => setFocused(false)}
      title={props.title}
    />
  );
}

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
              <HelpLink slug="unit-coordinate-drift" />
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
            <Show when={data.mouthpiece.fipple}>
              <HelpLink slug="flue-tsh-fipple-voicing" />
            </Show>
          </h3>
          <div class="grid grid-cols-2 gap-2">
            <NumberField
              label="Position"
              value={data.mouthpiece.position}
              onChange={(v) => {
                if (v != null) setData("mouthpiece", "position", v);
              }}
              step={0.001}
              displayPrecision={4}
            />
            <Show when={data.mouthpiece.fipple}>
              <NumberField
                label="Window Length"
                value={data.mouthpiece.fipple!.windowLength}
                onChange={(v) => {
                  if (v != null) setData("mouthpiece", "fipple", "windowLength", v);
                }}
                step={0.001}
                displayPrecision={4}
              />
              <NumberField
                label="Window Width"
                value={data.mouthpiece.fipple!.windowWidth}
                onChange={(v) => {
                  if (v != null) setData("mouthpiece", "fipple", "windowWidth", v);
                }}
                step={0.001}
                displayPrecision={4}
              />
              <NumberField
                label="Fipple Factor"
                value={data.mouthpiece.fipple!.fippleFactor}
                onChange={(v) => setData("mouthpiece", "fipple", "fippleFactor", v)}
                step={0.01}
                nullable
                displayPrecision={4}
              />
              <NumberField
                label="Window Height"
                value={data.mouthpiece.fipple!.windowHeight}
                onChange={(v) => setData("mouthpiece", "fipple", "windowHeight", v)}
                step={0.001}
                nullable
                displayPrecision={4}
              />
              <NumberField
                label="Windway Length"
                value={data.mouthpiece.fipple!.windwayLength}
                onChange={(v) => setData("mouthpiece", "fipple", "windwayLength", v)}
                step={0.001}
                nullable
                displayPrecision={4}
              />
              <NumberField
                label="Windway Height"
                value={data.mouthpiece.fipple!.windwayHeight}
                onChange={(v) => setData("mouthpiece", "fipple", "windwayHeight", v)}
                step={0.001}
                nullable
                displayPrecision={4}
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
            <HelpLink slug="bore-sac-body-geometry" />
          </h3>
          <table class="w-full text-xs border-collapse">
            <thead>
              <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Bore point label">
                  Name
                </th>
                <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Distance from top of bore">
                  Position
                </th>
                <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Internal bore diameter at this point">
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
                      <InlineNum
                        value={bp.borePosition}
                        onInput={(v) => setData("borePoint", i(), "borePosition", v)}
                      />
                    </td>
                    <td class="py-1 px-2">
                      <InlineNum
                        value={bp.boreDiameter}
                        onInput={(v) => setData("borePoint", i(), "boreDiameter", v)}
                      />
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
          <div class="flex gap-2 mt-2">
            <button
              class="px-2 py-0.5 rounded text-xs transition-colors"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              onClick={() => {
                const last = data.borePoint[data.borePoint.length - 1];
                const newPos = last ? last.borePosition + 1 : 0;
                const newDia = last ? last.boreDiameter : 1;
                setData("borePoint", produce((bp) => bp.push({ borePosition: newPos, boreDiameter: newDia })));
                sync();
              }}
              title="Add a new bore point at the end"
            >
              + Add Point
            </button>
            <button
              class="px-2 py-0.5 rounded text-xs transition-colors"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              disabled={data.borePoint.length <= 2}
              onClick={() => {
                setData("borePoint", produce((bp) => bp.pop()));
                sync();
              }}
              title={data.borePoint.length <= 2 ? "Minimum 2 bore points required" : "Remove the last bore point"}
            >
              - Remove Last
            </button>
          </div>
        </section>

        {/* Holes */}
        <section>
          <h3
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Holes
            <HelpLink slug="tone-holes-undercut-direction-holes" />
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
                  <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Tone hole label">
                    Name
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Distance from top of bore">
                    Position
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Distance from previous hole">
                    Spacing
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Hole opening diameter">
                    Diameter
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }} title="Tone hole chimney height">
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
                          <InlineNum
                            value={hole.borePosition}
                            onInput={(v) => setData("hole", i(), "borePosition", v)}
                          />
                        </td>
                        <td
                          class="py-1 px-2 text-right tabular-nums"
                          style={{ color: "var(--color-text-muted)" }}
                        >
                          {spacing()}
                        </td>
                        <td class="py-1 px-2">
                          <InlineNum
                            value={hole.diameter}
                            onInput={(v) => setData("hole", i(), "diameter", v)}
                          />
                        </td>
                        <td class="py-1 px-2">
                          <InlineNum
                            value={hole.height}
                            onInput={(v) => setData("hole", i(), "height", v)}
                          />
                        </td>
                      </tr>
                    );
                  }}
                </For>
              </tbody>
            </table>
          </Show>
          <div class="flex gap-2 mt-2">
            <button
              class="px-2 py-0.5 rounded text-xs transition-colors"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              onClick={() => {
                const last = data.hole[data.hole.length - 1];
                const newPos = last ? last.borePosition + 1 : (data.borePoint.length > 0 ? data.borePoint[data.borePoint.length - 1].borePosition * 0.5 : 5);
                const newDia = last ? last.diameter : 0.25;
                const newHeight = last ? last.height : 0.2;
                setData("hole", produce((h) => h.push({ borePosition: newPos, diameter: newDia, height: newHeight })));
                sync();
              }}
              title="Add a new tone hole at the end"
            >
              + Add Hole
            </button>
            <button
              class="px-2 py-0.5 rounded text-xs transition-colors"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              disabled={data.hole.length === 0}
              onClick={() => {
                setData("hole", produce((h) => h.pop()));
                sync();
              }}
              title={data.hole.length === 0 ? "No holes to remove" : "Remove the last hole"}
            >
              - Remove Last
            </button>
          </div>
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
              displayPrecision={4}
            />
          </div>
        </section>
      </div>
    </Show>
  );
}
