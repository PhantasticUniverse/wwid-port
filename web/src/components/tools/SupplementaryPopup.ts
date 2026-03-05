/** Supplementary acoustic info in a popup window (matches Java WIDesigner behavior). */

interface SupplementaryRow {
  note: string;
  freq: number;
  im_z_correction: number;
  air_speed?: number;
  air_flow_rate?: number;
  gain: number;
  q_factor: number;
}

interface SupplementaryResult {
  rows: SupplementaryRow[];
}

export function openSupplementaryPopup(result: SupplementaryResult, instrumentName: string) {
  const popup = window.open(
    "",
    `sup-${Date.now()}`,
    "width=780,height=520,menubar=no,toolbar=no,location=no,status=no"
  );
  if (!popup) {
    alert("Popup blocked — please allow popups for supplementary info.");
    return;
  }

  const doc = popup.document;
  doc.title = `Supplementary Info — ${instrumentName}`;

  // Check if any row has air speed / air flow data
  const hasAirSpeed = result.rows.some((r) => r.air_speed != null);
  const hasAirFlow = result.rows.some((r) => r.air_flow_rate != null);

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
      text-align: right; padding: 6px 10px;
      border-bottom: 1px solid #2e3345;
      color: #8b8fa3; font-weight: 600; font-size: 12px;
    }
    th:first-child { text-align: left; }
    td {
      padding: 4px 10px; text-align: right;
      border-bottom: 1px solid #1a1d27;
    }
    td:first-child { text-align: left; }
    .gain-good { color: #22c55e; }
    .gain-bad { color: #ef4444; }
  `;
  doc.head.appendChild(style);

  const h2 = doc.createElement("h2");
  h2.textContent = `Supplementary Info — ${instrumentName}`;
  doc.body.appendChild(h2);

  const table = doc.createElement("table");

  // Header
  const thead = doc.createElement("thead");
  const headerRow = doc.createElement("tr");
  const headers = ["Note", "Freq (Hz)", "Im(Z) Corr"];
  if (hasAirSpeed) headers.push("Air Speed (m/s)");
  if (hasAirFlow) headers.push("Air Flow");
  headers.push("Gain", "Q Factor");

  for (const label of headers) {
    const th = doc.createElement("th");
    th.textContent = label;
    headerRow.appendChild(th);
  }
  thead.appendChild(headerRow);
  table.appendChild(thead);

  // Body
  const tbody = doc.createElement("tbody");
  for (const row of result.rows) {
    const tr = doc.createElement("tr");

    addCell(doc, tr, row.note, "left");
    addCell(doc, tr, row.freq.toFixed(2));
    addCell(doc, tr, row.im_z_correction.toFixed(4));
    if (hasAirSpeed) addCell(doc, tr, row.air_speed != null ? row.air_speed.toFixed(2) : "—");
    if (hasAirFlow) addCell(doc, tr, row.air_flow_rate != null ? row.air_flow_rate.toFixed(2) : "—");

    const gainTd = addCell(doc, tr, row.gain.toFixed(4));
    gainTd.className = row.gain >= 1.0 ? "gain-good" : "gain-bad";

    addCell(doc, tr, row.q_factor.toFixed(1));

    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  doc.body.appendChild(table);
}

function addCell(doc: Document, tr: HTMLTableRowElement, text: string, align?: string): HTMLTableCellElement {
  const td = doc.createElement("td");
  td.textContent = text;
  if (align) td.style.textAlign = align;
  tr.appendChild(td);
  return td;
}
