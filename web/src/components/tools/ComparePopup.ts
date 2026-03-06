/** Compare instruments result in a popup window (matches Java WIDesigner behavior). */

interface CompareRow {
  category: string;
  field: string;
  old_value: number | null;
  new_value: number | null;
  difference: number | null;
  percent_change: number | null;
}

export interface CompareResult {
  old_name: string;
  new_name: string;
  rows: CompareRow[];
}

export function openComparePopup(result: CompareResult) {
  const popup = window.open(
    "",
    `compare-${Date.now()}`,
    "width=700,height=500,menubar=no,toolbar=no,location=no,status=no"
  );
  if (!popup) {
    alert("Popup blocked — please allow popups for comparison results.");
    return;
  }

  const doc = popup.document;
  doc.title = `Compare — ${result.old_name} vs ${result.new_name}`;

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
    th:nth-child(n+3) { text-align: right; }
    td {
      padding: 4px 10px;
      border-bottom: 1px solid #1a1d27;
    }
    td:nth-child(n+3) { text-align: right; font-family: monospace; }
    td.cat { color: #8b8fa3; }
    .pos { color: #22c55e; }
    .neg { color: #ef4444; }
  `;
  doc.head.appendChild(style);

  const h2 = doc.createElement("h2");
  h2.textContent = `${result.old_name} vs ${result.new_name}`;
  doc.body.appendChild(h2);

  const table = doc.createElement("table");

  // Header
  const thead = doc.createElement("thead");
  const headerRow = doc.createElement("tr");
  for (const label of ["Category", "Field", result.old_name, result.new_name, "Diff", "%"]) {
    const th = doc.createElement("th");
    th.textContent = label;
    headerRow.appendChild(th);
  }
  thead.appendChild(headerRow);
  table.appendChild(thead);

  // Body
  const tbody = doc.createElement("tbody");
  let lastCategory = "";
  for (const row of result.rows) {
    const tr = doc.createElement("tr");

    const showCat = row.category !== lastCategory;
    lastCategory = row.category;

    const tdCat = doc.createElement("td");
    tdCat.className = "cat";
    tdCat.textContent = showCat ? row.category : "";
    tr.appendChild(tdCat);

    const tdField = doc.createElement("td");
    tdField.textContent = row.field;
    tr.appendChild(tdField);

    const tdOld = doc.createElement("td");
    tdOld.textContent = row.old_value != null ? fmtNum(row.old_value) : "\u2014";
    tr.appendChild(tdOld);

    const tdNew = doc.createElement("td");
    tdNew.textContent = row.new_value != null ? fmtNum(row.new_value) : "\u2014";
    tr.appendChild(tdNew);

    const tdDiff = doc.createElement("td");
    tdDiff.textContent = row.difference != null ? fmtDiff(row.difference) : "\u2014";
    tr.appendChild(tdDiff);

    const tdPct = doc.createElement("td");
    if (row.percent_change != null) {
      tdPct.textContent = `${row.percent_change >= 0 ? "+" : ""}${row.percent_change.toFixed(2)}%`;
      tdPct.className = row.percent_change > 0 ? "pos" : row.percent_change < 0 ? "neg" : "";
    } else {
      tdPct.textContent = "\u2014";
    }
    tr.appendChild(tdPct);

    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  doc.body.appendChild(table);
}

function fmtNum(v: number): string {
  return Math.abs(v) < 0.001 ? v.toExponential(4) : v.toFixed(4);
}

function fmtDiff(v: number): string {
  const prefix = v >= 0 ? "+" : "";
  return prefix + fmtNum(v);
}
