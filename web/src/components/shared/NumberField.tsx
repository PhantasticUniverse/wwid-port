import { createSignal } from "solid-js";

/** Format a number to a given number of significant digits, stripping trailing zeros. */
function formatDisplay(value: number, sigDigits: number): string {
  if (value === 0) return "0";
  const s = Number(value.toPrecision(sigDigits));
  // Use toPrecision to format, then parseFloat to strip trailing zeros
  return String(s);
}

/** Labeled number input with on-blur commit and optional display formatting. */
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
  /** When set, show this many significant digits when not focused. */
  displayPrecision?: number;
  title?: string;
}) {
  const [local, setLocal] = createSignal(
    props.value != null ? String(props.value) : ""
  );
  const [focused, setFocused] = createSignal(false);

  // Sync from parent when value changes externally
  const updateFromProps = () => {
    setLocal(props.value != null ? String(props.value) : "");
  };

  const displayValue = () => {
    if (focused()) return local();
    if (props.value == null) return "";
    if (props.displayPrecision != null) {
      return formatDisplay(props.value, props.displayPrecision);
    }
    return String(props.value);
  };

  return (
    <div class={`flex items-center gap-2 ${props.class ?? ""}`}>
      {props.label && (
        <label
          class="text-xs whitespace-nowrap"
          style={{ color: "var(--color-text-muted)" }}
          title={props.title}
        >
          {props.label}
        </label>
      )}
      <input
        type={focused() ? "number" : "text"}
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
        value={displayValue()}
        onFocus={() => {
          updateFromProps();
          setFocused(true);
        }}
        onInput={(e) => setLocal(e.currentTarget.value)}
        onBlur={() => {
          setFocused(false);
          const v = local().trim();
          if (v === "" && props.nullable) {
            props.onChange(undefined);
          } else {
            const n = parseFloat(v);
            if (!isNaN(n)) props.onChange(n);
          }
        }}
        title={props.title}
      />
    </div>
  );
}

export { formatDisplay };
