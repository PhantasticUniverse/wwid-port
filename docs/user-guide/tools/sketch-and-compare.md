# Sketch & Compare

[User Guide](../index.md) > [Tools] > Sketch & Compare

The Sketch and Compare tools let you visualize instrument geometry. Sketch draws a single instrument's cross-section; Compare places two instruments side by side so you can see what changed after optimization or manual editing.

---

## Sketch

### Prerequisites

- An instrument file must be selected in the Study Panel.
- A tuning file is **not** required. Sketch works with just an instrument.

### Opening the Sketch

Click the **Sketch** button in the toolbar. A popup window opens showing the instrument cross-section. Your browser may need to allow popups for this site.


### Reading the Diagram

The sketch is an engineering-style cross-section drawing:

- **Bore profile**: Drawn as a dashed outline showing the bore diameter along the instrument's length. The bore is centered vertically, with the mouthpiece end at the left and the bell/open end at the right.
- **Center axis**: A thin dashed line runs along the bore center for reference.
- **Tone holes**: Shown as circles at their positions along the bore. Each hole is labeled with its name (or a number if unnamed). The circle diameter is proportional to the actual hole diameter.
- **Flange**: If the instrument has a flange (bell end), it appears as a vertical line at the bore's right end.
- **Axes**: The horizontal axis shows position along the instrument's length. The vertical axis shows width (bore diameter). Tick marks indicate dimensions in the instrument's length unit. Axis labels show "Length" and "Width" without explicit units.

### Mouthpiece Detail

The mouthpiece rendering depends on the study model:

- **Fipple flutes** (NAF and Whistle): The fipple window is drawn as a solid rectangle at the mouthpiece end. If windway dimensions are available, the windway is drawn as a dashed rectangle extending to the left of the window.
- **Transverse flutes** (Flute): The embouchure hole is drawn as an ellipse at its position on the bore.
- **Reed instruments** (Reed): No mouthpiece geometry is drawn, matching the original WIDesigner behavior.

### Summary Information

Below the diagram, a summary table shows:

- Bore length
- Number of tone holes
- Mouthpiece type
- Flange diameter

---

## Compare

### Prerequisites

- At least two instrument files must be loaded in the Study Panel.

If fewer than two instruments are loaded, the Compare button in the toolbar is disabled.

### Running a Comparison

1. Click the **Compare** button in the toolbar. A dialog opens with two dropdown selectors.
2. Choose the **Old (baseline)** instrument -- typically the original design.
3. Choose the **New (modified)** instrument -- typically the result of optimization.
4. Click **Compare**.

If you just ran an optimization, the two most recent instruments are pre-selected for convenience.


### Reading the Comparison

The comparison results open in a popup window showing a table with the following columns:

| Column | Description |
|--------|-------------|
| **Category** | Groups related dimensions together (e.g., Bore, Holes, Mouthpiece). Shown only on the first row of each group. |
| **Field** | The specific dimension being compared (e.g., bore point position, hole diameter). |
| **Old value** | The value in the baseline instrument. |
| **New value** | The value in the modified instrument. |
| **Diff** | The numerical difference (new minus old). Positive values are shown in green; negative in red. |
| **%** | The percent change relative to the old value. |


This table makes it easy to see exactly which bore points moved, which holes changed diameter or position, and by how much -- useful for understanding what the optimizer adjusted.

## See Also

- [Evaluation](evaluation.md) -- compare predicted tuning before and after changes
- [Optimization Workflow](../optimizers/workflow.md) -- use Sketch and Compare to verify optimization results
