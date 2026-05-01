import type { TuningResult, EvalRow } from "../../types/session";

/** Open evaluation results in a popup window (matches Java WIDesigner behavior). */
export function openEvalPopup(result: TuningResult, instrumentName: string): boolean {
  const popup = window.open(
    "",
    `eval-${Date.now()}`,
    "width=640,height=520,menubar=no,toolbar=no,location=no,status=no"
  );
  if (!popup) {
    return false;
  }

  const doc = popup.document;
  doc.title = `Evaluation — ${instrumentName}`;

  // Style
  const style = doc.createElement("style");
  style.textContent = `
    body {
      margin: 0; padding: 16px;
      background: #0f1117; color: #e4e6ef;
      font-family: "Inter", system-ui, -apple-system, sans-serif;
      font-size: 13px;
    }
    h2 { margin: 0 0 12px; font-size: 15px; font-weight: 600; }
    table { width: 100%; border-collapse: collapse; }
    th {
      text-align: left; padding: 6px 10px;
      border-bottom: 1px solid #2e3345;
      color: #8b8fa3; font-weight: 600; font-size: 12px;
    }
    th:not(:first-child) { text-align: right; }
    td { border-bottom: 1px solid #1a1d27; }
    tfoot td { padding: 8px 10px; font-weight: 600; border-top: 2px solid #2e3345; border-bottom: none; }
  `;
  doc.head.appendChild(style);

  // Heading
  const h2 = doc.createElement("h2");
  h2.textContent = `Evaluation — ${instrumentName}`;
  doc.body.appendChild(h2);

  // Table
  const table = doc.createElement("table");

  // Header
  const thead = doc.createElement("thead");
  const headerRow = doc.createElement("tr");
  for (const label of ["Note", "Target (Hz)", "Predicted (Hz)", "Deviation (cents)", "Weight"]) {
    const th = doc.createElement("th");
    th.textContent = label;
    headerRow.appendChild(th);
  }
  thead.appendChild(headerRow);
  table.appendChild(thead);

  // Body
  const tbody = doc.createElement("tbody");
  for (const row of result.rows) {
    tbody.appendChild(createEvalRow(doc, row));
  }
  table.appendChild(tbody);

  // Footer
  const tfoot = doc.createElement("tfoot");
  tfoot.appendChild(
    createSummaryRow(doc, "Net Error", result.net_error, true)
  );
  tfoot.appendChild(
    createSummaryRow(doc, "Mean Deviation", result.mean_deviation, false)
  );
  table.appendChild(tfoot);

  doc.body.appendChild(table);
  return true;
}

function createEvalRow(doc: Document, row: EvalRow): HTMLTableRowElement {
  const tr = doc.createElement("tr");
  const absCents = Math.abs(row.cents);
  const color = absCents < 5 ? "#22c55e" : absCents < 15 ? "#f59e0b" : "#ef4444";

  const cells = [
    { text: row.note, align: "left", mono: false, cellColor: "" },
    { text: row.target_freq.toFixed(2), align: "right", mono: false, cellColor: "" },
    { text: row.predicted_freq.toFixed(2), align: "right", mono: false, cellColor: "" },
    {
      text: `${row.cents >= 0 ? "+" : ""}${row.cents.toFixed(2)}`,
      align: "right",
      mono: true,
      cellColor: color,
    },
    { text: String(row.weight), align: "right", mono: false, cellColor: "" },
  ];

  for (const cell of cells) {
    const td = doc.createElement("td");
    td.textContent = cell.text;
    td.style.padding = "4px 10px";
    td.style.textAlign = cell.align;
    if (cell.mono) td.style.fontFamily = "monospace";
    if (cell.cellColor) td.style.color = cell.cellColor;
    tr.appendChild(td);
  }

  return tr;
}

function createSummaryRow(
  doc: Document,
  label: string,
  value: number,
  signed: boolean
): HTMLTableRowElement {
  const tr = doc.createElement("tr");

  const tdLabel = doc.createElement("td");
  tdLabel.colSpan = 3;
  tdLabel.textContent = label;
  tr.appendChild(tdLabel);

  const tdValue = doc.createElement("td");
  tdValue.style.textAlign = "right";
  tdValue.style.fontFamily = "monospace";
  tdValue.textContent = `${signed && value >= 0 ? "+" : ""}${value.toFixed(2)} cents`;
  tr.appendChild(tdValue);

  const tdEmpty = doc.createElement("td");
  tr.appendChild(tdEmpty);

  return tr;
}
