---
version: "1.0"
name: neuro
description: "Near-black neutral surfaces, signal-yellow emphasis, green identity and success, small 4–8px corners, and compact, clear information hierarchy."
colors:
  primary: "#d9ff38"
  success: "#22c55e"
  information: "#06b6d4"
  danger: "#f43f5e"
  canvas: "#06080d"
  surface: "#090c11"
  panel: "#0e1218"
  rail: "#070a0f"
  control: "#111720"
  control-hover: "#18222c"
  text: "#f7f8ef"
  text-muted: "#929a9f"
geometry:
  control-radius: 4px
  input-radius: 8px
  panel-radius: 8px
  primary-button-min-height: 36px
  compact-control-height: 28px
---

# neuro

neuro uses near-black neutral surfaces, signal-yellow emphasis, small corners, and continuous boundaries. Restrained color, clear hierarchy, and stable geometry organize information.

## 1. Theme definition

This specification defines color, typography, geometry, spacing, control states, and interaction expression. It does not prescribe a product category, business domain, page purpose, information architecture, or technical framework.

Choose page structure and controls for the actual content. Apply the relevant rules to existing functionality; do not add a layout or feature merely because a theme example includes it. Preserve business fields, permissions, workflows, and content meaning.

## 2. Design principles

1. **Task before chrome.** Keep the current resource, action, and result visible. A panel exists to establish an ownership or scrolling boundary.
2. **One primary action per task context.** Signal yellow identifies the main action or current state. It is not a decorative wash over every heading and icon.
3. **Continuous surfaces, selective grouping.** Establish relationships with alignment, spacing, hairlines, and necessary containers rather than treating different levels as equally important cards.
4. **Neutral surfaces carry the interface.** Black, gray, and text dominate; semantic color supplies small, meaningful signals.
5. **Readable at rest.** Controls, secondary text, disabled states, and permissions remain legible without hovering.
6. **Density is contextual.** A 28px settings control and a 36px primary action can coexist. Do not average them into one mandatory size.

### Interface copy

**No decorative English headings.** Keep only titles that identify the current page, object, or real group, using natural conventions for the target language. Do not add English eyebrows, all-caps micro-labels, slash-separated pseudo-hierarchies, or slogan subtitles for visual effect. This does not prohibit English interfaces, technical abbreviations, or real identifiers.

**No explanatory copy by default.** If a button, field, or title already communicates its meaning, do not append purpose descriptions, tutorials, duplicate labels, or design slogans. Improve naming, grouping, and state first; do not compensate for an unclear control with a paragraph.

Keep a short hint only when omission would cause ambiguity, failure, or risk: non-obvious input formats/units, errors and recovery, permission limits, destructive consequences, or essential demo boundaries. Show it where and when needed, rather than reserving a helper row for every control. Preserve icon accessible names and necessary field labels. Specifications and component atlases may retain essential technical annotations; do not copy those annotations into permanent product-interface explanations.

## 3. Palette and semantic roles

### Dark mode — default

| Role | Value | Usage |
|---|---|---|
| Canvas | `#06080d` | Outer application background |
| Rail | `#070a0f` | Navigation and titlebar |
| Surface | `#090c11` | Continuous working board |
| Panel | `#0e1218` | Related content, menus, dialogs |
| Control | `#111720` | Inputs and secondary actions |
| Control hover | `#18222c` | Raised control state |
| Text | `#f7f8ef` | Main labels and content |
| Muted text | `#929a9f` | Explanatory text and metadata |
| Hairline | text at 12% opacity | Boundaries, not heavy boxes |
| Signal yellow | `#d9ff38` | Primary action, active item, focus, required attention |
| Signal green | `#22c55e` | Brand identity, online, success, completion |
| Information blue | `#06b6d4` | Links, technical information, synchronization |
| Danger red | `#f43f5e` | Error, destructive action, high risk |

There is one base yellow and one base green. Derive states rather than introducing competing brand shades:

~~~css
--primary-hover: color-mix(in srgb, #d9ff38 86%, white);
--primary-pressed: color-mix(in srgb, #d9ff38 82%, black);
--on-primary: color-mix(in srgb, #d9ff38 14%, black);
--signal-soft: color-mix(in srgb, #d9ff38 12%, #0e1218);
--border: color-mix(in srgb, #f7f8ef 12%, transparent);
--signal-border: color-mix(in srgb, #d9ff38 42%, var(--border));
~~~

Use labels, icons, checkmarks, edge markers, or structure alongside color. Green is not the default primary-button fill. Ordinary secondary actions use neutral controls, not blue by default.

### Data changes are not outcome states

An increase/decrease is not automatically success/failure. For latency, cost, memory, or throughput, show the signed change or direction with its unit and comparison basis; decide whether that direction is beneficial in the actual business context. Increased memory is not a successful task, and decreased latency is not a failure. Use neutral text when no favorable/unfavorable meaning is defined. Do not add a universal “up = green, down = red” rule or competing trend colors.

### Small inverse surfaces, not mandatory white cards

Use `#f3f5f1` with `#20252b` text for small inverse parts such as tooltips, switch thumbs, or marks that need a separate backing. Main containers follow the neutral surface ladder; do not use large white cards to express selection or focus.

## 4. Typography

UI stack: `"Bahnschrift", "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif`.

Technical stack: `"Cascadia Mono", "JetBrains Mono", Consolas, monospace`. Font names are fallback preferences, not bundled assets.

| Role | Practical baseline | Treatment |
|---|---|---|
| Application page title | 20–24px | 600–650; not an oversized hero |
| Section title | 14–16px | 600–650 |
| Body | 13–14px | Normal weight; readable line height |
| Standard action | 13px, weight 800 | Strong but short |
| Compact setting/tab | 12px | Approximately 550–600 |
| Supporting metadata | 11–12px | Muted, with sufficient contrast |
| Identifier, code, size | 11–13px | Monospace, tabular numerals |

These values are default typography baselines. Support platform font fallbacks, user text-size preferences, and scaling; fixed pixel values must not prevent readability adjustments.

Use natural case and spacing for interface titles and group labels. Do not manufacture visual style with all-caps, monospace eyebrows, or wide tracking. Do not use a marketing display typeface for dense settings.

## 5. Geometry and spacing

| Element | Rule |
|---|---|
| Buttons / navigation | 4px corner radius |
| Inputs / common panels / dialogs | 8px corner radius |
| Rail / board | Square, continuous boundaries; no floating-card shadow |
| Standard button | Minimum 36px high; 12px horizontal padding |
| Regular input | 10px vertical / 12px horizontal padding; height follows text metrics |
| Compact settings input/button | 28px high; 12px text; 8–10px horizontal padding |
| Typical panel | 18px padding |
| Workspace content | 14px gap; roughly 16–20px outer padding |
| Dialog | 24px padding, 16px content gap |
| Border | Normally 1px |
| Small surface shadow | `0 2px 8px rgb(0 0 0 / 14%)` |

Use the small 4/8px family by default. The rounded-track switch is an explicit control variant; do not extend its pill geometry to buttons, inputs, or all containers.

## 6. Layout grammar

Content and task determine page structure; no fixed combination of rail, titlebar, board, or details is required. When these regions are needed, use continuous boundaries, compact grouping, and explicit scroll ownership.

A navigation layout can start with a **212px rail, 38px context bar, 30px navigation rows, and a minimum 46px content header**, then adjust for text, actions, and available width. These are optional layout baselines, not a required page skeleton or window size.

- Give the main task most of the usable width.
- Keep search, filters, view controls, and contextual actions together.
- Prefer stable rows for data. Use grids only when the content benefits from visual previews.
- A detail inspector should expose properties without turning every property into another card.
- Make long lists and inspectors scroll within clear bounds; do not let one disclosure resize the whole shell.
- At narrow widths, collapse or move navigation and stack details. Do not shrink all text.
- Do not invent a persistent native-window control strip in a normal browser website.

### Long tables

For a long table that requires comparison across rows, keep column headers visible while its body scrolls within the existing bounded region. Give sticky headers an opaque surface, correct stacking, and aligned columns. At narrow widths, choose meaningful column priorities or local horizontal scrolling rather than shrinking all text or converting every row into a large card. This is a long-table variant, not a requirement to add persistent headers or extra scrolling containers to short lists; freezing the first column is a separate, task-specific choice.

### Content-preview fidelity

Screenshots, diagrams, charts, and captured content are user content, not theme decoration. Inspection/editing previews preserve aspect ratio and essential edges; apply theme color to surrounding controls, boundaries, and selection markers, not as an arbitrary tint, stretch, or rotation of the content itself. Browsing thumbnails may crop deliberately when that serves recognition; inspection previews normally show the complete content. Choose fit/zoom behavior for the task rather than applying `cover` or `contain` universally. Neutral letterboxing is preferable to distortion or hidden technical details.

## 7. Component recipes

| Component | Visual recipe and behavior |
|---|---|
| Primary button | Yellow fill, dark text, 4px corners. Derived hover/pressed shades. Only one per task context. |
| Secondary / outline | Neutral control fill, hairline, main text. Hover raises the surface and border, not the semantic importance. |
| Quiet / icon action | Transparent surface; labeled icon; visible hover and keyboard focus. Compact 26–28px shell actions are legitimate. |
| Destructive action | Red text, red-derived line and soft fill. Confirm meaningful deletion in an in-page dialog. |
| Disabled control | Muted text, control surface, stable border, no competing shadow; not a globally faded unreadable element. |
| Input / textarea / select | Visible border and label, control fill, appropriate text/placeholder, yellow focus. Error has a red boundary and nearby explanation. |
| Checkbox / radio / range | Yellow semantic accent with native keyboard operation; the control itself conveys state. |
| Switch | Rounded track is a supported exception; the small inverse thumb remains readable in both modes. |
| Navigation | Active item has a yellow-derived surface/border and an inset edge marker; icon and label remain distinguishable. |
| Tabs | Compact neutral items; current item has a signal-soft fill and strong border. A tablist switches content panels and supports arrow navigation. |
| Panel / card | Dark panel, 8px corners, subtle shadow when useful. Continuous boards are not rounded floating cards. |
| Data rows | Clear columns and stable spacing; selection uses a control/marker plus a surface change. Set row height for content density and keep it consistent within a view. |
| Status / notification | Short text and icon, semantic accent, neutral/soft surface. A toast must not be the only record of a persistent error. |
| Tooltip | Small inverse surface, dark text in dark mode; contextual and nonessential to understanding the control. |
| Dialog / menu | Bounded to the viewport, correct stacking and internal scrolling. Dialog actions align to the task, usually at the end. |
| Empty / loading / failure | State the reason or next action. Do not show fake connected, uploaded, or successful states. |

Reuse the target project's existing controls and apply these visual and interaction rules. No particular framework, component library, or CSS class name is required.

### Choosing selection controls

| Context | Preferred form | Rule |
|---|---|---|
| A few short, mutually exclusive options | Segmented single choice | For example Off/On or 100%/110%/125%/150%. Exactly one option per group, with native radio semantics, not tabs or independent switches. |
| Many options, long labels, or insufficient horizontal space | Single-select dropdown | Prefer `select`; do not squeeze the interface to expose every option. |
| A small multiple-choice set | Flat checkbox group | Arrange short labels horizontally, longer labels or narrow regions vertically. Each option is independent; multiple or no selections are allowed. |
| Many multiple-choice options or limited space | Checkbox dropdown | Open a checkbox list, not a single-choice menu. Keep it open while checking, show a count in a fixed single-line trigger, and scroll inside the panel. |

Use a neutral control track with signal-yellow-derived fill/border/inset marking for the selected segment and the default 4px control radius. Checkboxes and checkbox dropdowns use the same theme semantics.

All selection states preserve geometry: checking must not change padding, border thickness, text size/weight, column width, or row height. Use inset shadows/outlines or preallocated borders for selection markers. Patch selection properties and styles rather than rebuilding a table per click. Put batch actions in an existing region instead of inserting a row that pushes the list down.

### Copyable technical values

For frequently copied IDs, endpoints, paths, commands, or configuration snippets, place a low-emphasis copy action beside the value. Do not add it to every piece of text. A single value needs no extra code-card shell; a multiline snippet can have its own code container. Use the existing 4px action geometry and keep the value readable/selectable.

- Keep the action slot stable. Briefly replace its icon with an in-place success mark after a real successful copy; do not shift the value, add “click to copy” helper text, or emit a toast for every routine copy.
- Define whether the copied value is complete or displayed/masked. Copying must not expose secrets hidden by product permissions. Report an actual failure briefly where it occurs; never display a success mark for a failed operation.
- Give the action an accessible name and keyboard operation, and announce success/failure without depending on the icon alone. Verify clipboard support in the target host, including `file://` when local web pages are supported.

### Compact numeric steppers

For values that need repeated fine adjustment, group decrement, value, and increment in one compact property row. Allow direct editing when useful and keep real units such as `px` or `%` with the value. Use the 28px contextual-control baseline where appropriate, 4px action corners, and the existing input geometry; ordinary forms can retain their normal readable size. Do not replace every native number input or slider, and do not add duplicate step buttons to an already sufficient control.

- Define range, step, precision, empty-value semantics, and boundary/out-of-range behavior from the product. Disable only the relevant increment/decrement action at a limit; do not silently substitute an arbitrary default.
- Retain native number-input semantics where suitable, direct input and keyboard operation, and accessible names for decrement/increment. The group must not depend on pointer-only mini buttons.
- Reserve enough width for expected digits and units. Incrementing, crossing a digit boundary, focusing, or disabling an action must not resize the group or push adjacent properties. Units are field information, not an extra explanatory paragraph.

### Core control state recipes

Use these states according to each control's actual behavior; not every control needs every state. Keep hover, pressed, keyboard focus, selection, and disabled meanings distinct. Focus can coexist with selection or an error.

| Control | At rest | Hover / pressed | Persistent state or error | Keyboard focus | Disabled / loading |
|---|---|---|---|---|---|
| Action button | Its primary, neutral, or destructive recipe | Derived surface/border; primary uses yellow-derived hover/pressed shades | Pressed styling only for a real toggle, not a completed one-shot action | Visible 2px signal outline | Readable muted control; block unavailable actions; reserve feedback space for real loading |
| Text / number / select | Control fill, visible border and label | No movement or padding change | Read-only stays selectable; invalid has danger boundary plus a necessary nearby error | Signal boundary and visible outline; an error remains identifiable | Readable but unavailable; read-only is not disabled |
| Checkbox / radio / switch | Recognizable native or equivalent control | Preserve hit area and geometry | Checkmark, radio dot, or thumb position plus signal treatment | Focus remains visible on the actual control | Unavailable without erasing its current value; do not invent a generic loading state |
| Segmented radio | Neutral track; reserved borders and equal treatment | Control-hover surface; stable text weight | Signal-soft fill, signal border and inset mark for the selected option | Outline on the visible segment | Readable muted option; no selection change through a disabled option |
| Data row | Stable columns and neutral surface | Surface-only hover | Checkbox/marker plus selected surface or inset edge | Visible on the actual interactive cell/control | Disable only unavailable actions; retain row content |
| Menu / checkbox dropdown | Neutral trigger; panel closed | Interactive-row hover without displacement | Open trigger has signal boundary; checked options retain checkmarks and soft fill | Visible trigger/option focus; Escape returns to trigger | Unavailable items cannot execute; do not confuse no results with loading |

Borders, padding, type size/weight, row heights, and feedback slots remain geometrically stable through these states. Verify the combinations actually used in dark/light modes and with reduced motion. Do not invent control behavior to fill the matrix.

Use a short, truthful empty-state title. Add an action or missing prerequisite only when necessary, not a slogan or repeated operating instructions. Page-level empty states may be centered with moderate reserved height. Cards, scope bands, and embedded lists use a compact inline empty state rather than a page-sized minimum; it must not expand a business card or displace fixed actions.

## 8. Interaction contract

- Support visible 2px keyboard focus, normal Tab order, and semantic labels.
- A modal supports Escape, initial focus, trapped keyboard focus, background scroll locking, and restoration to its trigger or a surviving equivalent.
- Menus close on outside interaction and Escape. Actions remain keyboard reachable.
- A tablist supports Left/Right and Home/End; only the active tab is in the normal tab sequence.
- Segmented choices retain native radio arrow navigation and a single Tab entry; checkboxes use Tab/Space. Checkbox dropdowns open with Enter/Space, expose options via Tab and Space, and close on Escape with trigger focus restored or on outside interaction. The expanded panel is out of normal document flow.
- Hover mainly changes surface and border. Do not push adjacent elements or animate the entire layout.
- Short state transitions are approximately 140–200ms. Respect `prefers-reduced-motion`.
- Long names wrap or truncate deliberately without hiding the only identifier or action.
- Selection, error, and progress need a non-color explanation.
- State feedback matches actual operation results. Never present an unperformed or failed operation as successful.

## 9. Light mode

| Role | Light value |
|---|---|
| Canvas / surface / panel | `#edf1e4` / `#f8faf3` / `#ffffff` |
| Rail | `#e4e9dc` |
| Control / hover | `#f1f4ea` / `#e7ecdf` |
| Text / muted | `#10150e` / `#5d6659` |
| Hairline | Text at 16% opacity |

Keep the base signal colors, but darken signal **text** through mixing with ink: yellow 34%, green and blue 52%, red 68%. Primary yellow fills still use dark text. Inverse tooltip surfaces flip to `#1b2318` with `#f4f7ef` text. Change the palette only; preserve density, geometry, and information hierarchy.

## 10. Motifs and exclusions

Triangles, clipped corners, diagonal lines, grids, and rings can provide **local structural meaning**, such as a connection diagram or meaningful marker. They are not required wallpaper.

Avoid purple/pink replacement palettes, glow orbs, global glass blur, oversized rounded cards, all-pill controls, redundant nested boxes, and equal-weight metric-card walls. Restrained surface shadows, blurred modal backdrops, and rounded-track switches are allowed. Local effects must serve hierarchy or control meaning, not spread into decoration.

## 11. Ready-to-use implementation brief

> Copy constraint: no decorative English eyebrows, all-caps micro-labels, or slogan subtitles. Do not add explanatory copy by default or explain self-evident buttons and fields. Retain only short hints essential to understanding the task or preventing error/risk, necessary field labels, and accessible names.

> Selection constraint: use themed segmented radios for a few short single-choice options and prefer dropdowns for larger or non-fitting sets. For multiple choice, use horizontal/vertical checkbox groups or a dropdown containing checkboxes. Checking changes state only, not control geometry, table identity, or list position.

> Detail constraint: where the task needs them, use in-place technical-value copying with truthful feedback and compact numeric steppers with explicit units, limits, and stable width. Keep long-table headers visible within local scrolling; preserve the fidelity of inspection previews. Do not equate data direction with success/failure. Follow the applicable control state recipes in both palette modes. Do not add unnecessary controls or business capabilities just to satisfy these recipes.

> Apply the neuro theme to the target interface without changing its business definition or feature scope. Use the exact near-black neutral surface ladder, signal-yellow primary/active/focus states, green identity/success, cyan information, and red danger. Default to 4px controls and 8px inputs/panels, with 36px normal buttons and 28px contextual settings controls. Choose layout for the actual content, using continuous boundaries and compact grouping; do not require a rail, inspector, or metric cards. Use the specified UI font stack and monospace technical values. Do not add large white focus cards, glass wallpaper, or decorative neon. Preserve business fields, permissions, and actual operation results. Implement necessary labels, keyboard focus, Escape, modal focus restoration, empty/error/loading states, responsive scrolling, and reduced motion. Reuse project components without assuming a particular framework.

## 12. Acceptance checklist

- [ ] Information hierarchy is clear; the theme has not changed the product definition, business fields, or feature scope.
- [ ] The exact base colors and their semantic roles are preserved.
- [ ] Primary, secondary, danger, disabled, selected, and focus states differ clearly.
- [ ] 4/8px geometry is the default, with justified local exceptions.
- [ ] Normal and compact controls have intentional, documented contexts.
- [ ] Inverse surfaces are limited to suitable small parts; large white cards do not replace focus or selection markers.
- [ ] No decorative English headings, slogans, or unnecessary explanations; necessary labels, errors, and risk information remain intact.
- [ ] Single/multiple-choice forms fit the option set and available space; checking preserves control geometry, table columns, and list position.
- [ ] Where copying is offered, its scope, permissions, truthful in-place feedback, stable geometry, and host clipboard behavior are verified.
- [ ] Where numeric steppers are used, units, range, step, precision, empty/boundary behavior, keyboard operation, and width are intentional.
- [ ] Long comparative tables retain visible aligned headers within bounded scrolling; short lists gain no unnecessary chrome.
- [ ] Inspection previews preserve content and aspect ratio; thumbnail cropping is deliberate and the theme does not distort the content.
- [ ] Data-change direction is not confused with success/failure; units and comparison basis are clear.
- [ ] Applicable control states and their combinations follow the recipe matrix without geometry changes and are checked in dark/light modes.
- [ ] Keyboard, Escape, long text, narrow layouts, scrolling, and light mode work.
- [ ] Controls with the same semantic role remain consistent across pages.
- [ ] State feedback matches actual operation results.
