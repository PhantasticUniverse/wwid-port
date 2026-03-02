import { createSignal } from "solid-js";

/** Labeled number input with on-blur commit. */
export default function NumberField(props: {
  label?: string;
  value: number | undefined;
  onChange: (value: number | undefined) => void;
  step?: number;
  min?: number;
  max?: number;
  class?: string;
  disabled?: boolean;
  nullable?: boolean;
}) {
  const [local, setLocal] = createSignal(
    props.value != null ? String(props.value) : ""
  );

  // Sync from parent when value changes externally
  const updateFromProps = () => {
    setLocal(props.value != null ? String(props.value) : "");
  };

  return (
    <div class={`flex items-center gap-2 ${props.class ?? ""}`}>
      {props.label && (
        <label class="text-xs whitespace-nowrap" style={{ color: "var(--color-text-muted)" }}>
          {props.label}
        </label>
      )}
      <input
        type="number"
        step={props.step ?? "any"}
        min={props.min}
        max={props.max}
        disabled={props.disabled}
        class="w-full px-2 py-1 rounded text-xs text-right tabular-nums"
        style={{
          background: "var(--color-surface-alt)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text)",
        }}
        value={local()}
        onFocus={updateFromProps}
        onInput={(e) => setLocal(e.currentTarget.value)}
        onBlur={() => {
          const v = local().trim();
          if (v === "" && props.nullable) {
            props.onChange(undefined);
          } else {
            const n = parseFloat(v);
            if (!isNaN(n)) props.onChange(n);
          }
        }}
      />
    </div>
  );
}
