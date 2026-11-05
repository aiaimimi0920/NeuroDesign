---
version: "1.0"
name: neuro-mini
description: "Near-black neutral surfaces, signal-yellow emphasis, square primitives, compact spacing, minimal container overhead, and a bounded floating-toolbar variant."
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
  default-radius: 0px
  tool-height: 24px
  recommended-web-control-height: 28px
  floating-strip-radius: 9px
  floating-strip-button-radius: 5px
---

# neuro-mini

neuro-mini uses near-black neutral surfaces, signal-yellow emphasis, square primitives, and compact spacing. Continuous rows, inline properties, and few containers reduce the interface footprint while preserving readability and complete states.

## 1. Theme definition

This specification defines color, typography, geometry, density, control states, and interaction expression. It does not prescribe a product category, business domain, page purpose, information architecture, or technical framework.

Compactness comes from restrained structure, containers, and spacing, not global scaling, unreadably small text, or removed meaning. Choose layouts and controls for the actual content. Preserve business fields, permissions, workflows, and content meaning; do not add example features merely to apply the theme.

## 2. Design principles

1. **Available space is a resource.** Every permanent bar, label, gap, and container needs a task-related reason to exist.
2. **Compress structure before text.** Remove duplicate headings, nested boxes, and oversized padding before reducing legibility.
3. **Square by default, exceptions by purpose.** Flat edges let rows and controls meet without decorative gutters.
4. **Put properties beside the task.** Use compact groups, icon tools, inline values, and disclosure instead of a full page for every adjustment.
5. **Do not remove meaning.** Labels, tooltips, error explanations, selection, and keyboard focus remain available.
6. **Adjust density for content.** A settings dialog or long text may be more spacious than a toolbar. Not every control must be 24px.

### Interface copy

**No decorative English headings.** Keep only titles that identify the current page, object, or real group, using natural conventions for the target language. Do not add English eyebrows, all-caps micro-labels, slash-separated pseudo-hierarchies, or slogan subtitles for visual effect. This does not prohibit English interfaces, technical abbreviations, or real identifiers.

**No explanatory copy by default.** If a button, field, or title already communicates its meaning, do not append purpose descriptions, tutorials, duplicate labels, or design slogans. Improve naming, grouping, and state first; do not compensate for an unclear control with a paragraph.

Keep a short hint only when omission would cause ambiguity, failure, or risk: non-obvious input formats/units, errors and recovery, permission limits, destructive consequences, or essential demo boundaries. Show it where and when needed, rather than reserving a helper row for every control. Preserve icon accessible names and necessary field labels. Specifications and component atlases may retain essential technical annotations; do not copy those annotations into permanent product-interface explanations.

## 3. Palette and semantics

### Dark mode — default

| Role | Value | Usage |
|---|---|---|
| Canvas | `#06080d` | Outer tool surface |
| Rail | `#070a0f` | Compact navigation or support chrome |
| Surface | `#090c11` | Main work area |
| Panel | `#0e1218` | Properties, menus, dialogs |
| Control | `#111720` | Fields and neutral actions |
| Control hover | `#18222c` | Readable interaction state |
| Text | `#f7f8ef` | Content and controls |
| Muted text | `#929a9f` | Secondary information |
| Hairline | `rgba(255, 255, 255, 0.12)` | Small structural boundaries |
| Signal yellow | `#d9ff38` | Active tool, focus, primary task, attention |
| Signal green | `#22c55e` | Identity, online, success, completion |
| Information blue | `#06b6d4` | Links, technical information, synchronization |
| Danger red | `#f43f5e` | Failure and destructive actions |

No second yellow or green is needed to make the theme feel richer. Active surfaces and colored borders are derived with transparency or `color-mix`.

~~~css
--active-background: color-mix(in srgb, #d9ff38 14%, transparent);
--active-border: color-mix(in srgb, #d9ff38 38%, transparent);
--text-secondary: color-mix(in srgb, #f7f8ef 78%, transparent);
--panel-raised: color-mix(in srgb, #111720 98%, transparent);
~~~

Menus, dialogs, and sticky headers need stable, readable backgrounds. Do not depend on desktop transparency or glass blur. Where an opaque active surface is needed, mix 12% signal yellow into the panel color rather than introducing a new accent.

Use non-color signals too: a pressed tool, checkbox, inset edge, state label, or icon. Do not make green the default main action.

### Data changes are not outcome states

An increase/decrease is not automatically success/failure. For latency, cost, memory, or throughput, show the signed change or direction with its unit and comparison basis; decide whether that direction is beneficial in the actual business context. Increased memory is not a successful task, and decreased latency is not a failure. Use neutral text when no favorable/unfavorable meaning is defined. Do not add a universal “up = green, down = red” rule or competing trend colors.

## 4. Typography

For regular tool and settings text, prefer `system-ui, "Segoe UI", "Microsoft YaHei", sans-serif`. For values and identifiers, use `ui-monospace, "SFMono-Regular", Consolas, monospace`.

| Role | Working baseline | Treatment |
|---|---|---|
| Compact work-area title | 14–16px | 600; avoid a separate tall hero |
| Group heading | 12–14px | 600 |
| Body / standard field | 12px | Readable line height, around 1.4–1.5 |
| Dense value / helper | 11px | Monospace where appropriate |
| Long-form settings title | 20px | Keep reading-oriented areas legible |
| Small tool icon | 14px | Secondary property controls |
| Floating-toolbar icon | 22.5px | Only for the named floating-toolbar variant |

Small metadata is secondary, not the only explanation of an action. Use natural case and spacing for interface titles and group labels, without decorative English eyebrows. Do not impose a condensed display font, letter-spacing trick, or smaller global scale to manufacture density.

Use these typography baselines according to content density rather than compressing every role to one size. Support platform font fallbacks, user text-size preferences, and scaling.

## 5. Geometry, density, and exceptions

### Default primitives

The base corner radius is **`0px`** and backdrop blur is **`0px`**. Shells, common inputs, menus, list items, and property controls remain square.

| Context | Rule |
|---|---|
| Compact property icon action | 24px-wide control and approximately 14px icon |
| Switch | 42 × 24px, square 14px thumb |
| Standard field/action | Default 28px baseline |
| Compact web input | 4px vertical / 8px horizontal padding |
| Group gap | Typically 4–8px |
| Panel padding | Around 8–12px for compact properties |
| Border | Usually 1px; avoid a second outer frame around every group |
| Compact menu | Normally at least 24px per row; a dedicated pointer-tool menu may use 20px |
| Reading/settings field | Minimum 36px, 8px vertical / 10px horizontal padding |
| Reading/settings dialog | Content padding 20px, 16px gaps; header/footer 18px vertical / 20px horizontal |
| Compact panel shadow | None by default; floating layers can retain restrained separation |

Set data-row height for content and density, keeping it consistent within a view. Do not extend a dedicated menu's 20px row height to every input or touch target.

### Floating-toolbar variant

A standalone floating toolbar may use a **9px outer radius**, **5px tool-button radius**, and **4px menu radius**, with **4px vertical / 5px horizontal outer padding** and no shadow. Menu rows remain square.

Apply these corners only to the **floating-toolbar variant**, not ordinary forms, lists, or panels. The square primitive default does not remove this variant.

## 6. Layout grammar

- Keep the primary content dominant; permanent chrome must earn its footprint.
- Use short single-line tool groups where possible. Merge repeated identity labels instead of removing important state.
- Prefer continuous rows and direct property lists over card-per-property layouts.
- Reveal advanced fields on demand. Keep common controls near their affected object.
- Separate a narrow tool surface from a reading surface: compactness is not enforced by making a long form unreadable.
- Use clear local scrolling for lists and property panes.
- At narrow widths, wrap tool groups and stack/collapse navigation. Do not apply `transform: scale(...)` to the application.
- Touch contexts may need larger hit areas; desktop compact dimensions are not a reason to remove keyboard or pointer accessibility.

Content determines page structure; no fixed combination of rail, toolbar, or inspector is required. Size any such regions for labels, actions, and available width. Do not reserve a permanent bar for decorative copy or mandate a fixed window size.

### Long tables

Only when content genuinely needs a long, multicolumn comparison, keep column headers visible while its body scrolls within the existing bounded region. Give sticky headers an opaque surface, correct stacking, and aligned columns. At narrow widths, choose meaningful column priorities or local horizontal scrolling rather than shrinking text or turning every row into a large card. Small lists need no extra permanent header or scrolling container merely to satisfy this variant; freezing the first column is a separate, task-specific choice.

### Content-preview fidelity

Screenshots, diagrams, charts, and captured content are user content, not theme decoration. Inspection/editing previews preserve aspect ratio and essential edges; apply theme color to surrounding controls, boundaries, and selection markers, not as an arbitrary tint, stretch, or rotation of the content itself. Browsing thumbnails may crop deliberately when that serves recognition; inspection previews normally show the complete content. Choose fit/zoom behavior for the task rather than applying `cover` or `contain` universally. Neutral letterboxing is preferable to distortion or hidden technical details; do not sacrifice content fidelity to make every preview fill a compact rectangle.

## 7. Component recipes

| Component | Visual recipe and behavior |
|---|---|
| Primary action | Yellow fill with dark text, square edge. Reserve it for the current task, not every toolbar icon. |
| Neutral action | Control or transparent background, hairline, readable text; short padding. |
| Icon / compact toggle | Small square target, consistent icon alignment, pressed-state surface and border; supply an accessible name. |
| Floating tool | Use the named floating-toolbar variant; active tool has a small inset signal outline. |
| Destructive action | Red-derived line/text/soft fill. When confirmation is needed, identify the target, consequences, and cancellation action. |
| Disabled control | Muted but readable, fixed border, no glow. It remains identifiable as a control. |
| Input / number / select | Square boundary, compact padding, legible value. Group icon + value where that saves a redundant row. |
| Switch | Square track and thumb; state changes color and thumb position. Use a real keyboard-operable control. |
| Checkbox / radio / slider | Yellow accent, visible value/selection, native or equivalent keyboard behavior. |
| Navigation / tabs | Flat, short items with a clear current marker; avoid oversized rail blocks. |
| Panel / row | Square, neutral, little or no shadow. Share separators rather than stacking frames. |
| Status badge | Small label and semantic signal; do not rely on a colored dot alone. |
| Tooltip / menu | Useful context without a permanent label tax. Ordinary tooltips do not contain essential actions. |
| Dialog | Compact relative to its task, but with readable text, bounded scrolling, close/cancel and clear confirmation. |
| Empty / loading / error | Preserve the reason and next step. A smaller surface is not an excuse to omit state. |

Choose controls for actual functionality and reuse the target project's components. This specification does not require a particular framework, component library, or CSS class name.

### Choosing selection controls

| Context | Preferred form | Rule |
|---|---|---|
| A few short, mutually exclusive options | Segmented single choice | For example Off/On or 100%/110%/125%/150%. Exactly one option per group, with native radio semantics, not tabs or independent switches. |
| Many options, long labels, or insufficient horizontal space | Single-select dropdown | Prefer `select`; do not squeeze the interface to expose every option. |
| A small multiple-choice set | Flat checkbox group | Arrange short labels horizontally, longer labels or narrow regions vertically. Each option is independent; multiple or no selections are allowed. |
| Many multiple-choice options or limited space | Checkbox dropdown | Open a checkbox list, not a single-choice menu. Keep it open while checking, show a count in a fixed single-line trigger, and scroll inside the panel. |

Use a neutral control track with signal-yellow-derived fill/border/inset marking for the selected segment. Both track and options remain square, without the floating-toolbar corner variant. Checkboxes and checkbox dropdowns use the same theme semantics.

All selection states preserve geometry: checking must not change padding, border thickness, text size/weight, column width, or row height. Use inset shadows/outlines or preallocated borders for selection markers. Patch selection properties and styles rather than rebuilding a table per click. Put batch actions in an existing region instead of inserting a row that pushes the list down.

### Copyable technical values

For frequently copied IDs, endpoints, paths, commands, or configuration snippets, place a low-emphasis copy action beside the value. Do not add it to every piece of text. Prefer one compact row rather than another label or code-card shell; a multiline snippet can have its own code container. Keep the default square action geometry and the value readable/selectable.

- Keep the action slot stable. Briefly replace its icon with an in-place success mark after a real successful copy; do not shift the value, add “click to copy” helper text, or emit a toast for every routine copy.
- Define whether the copied value is complete or displayed/masked. Copying must not expose secrets hidden by product permissions. Report an actual failure briefly where it occurs; never display a success mark for a failed operation.
- Give the action an accessible name and keyboard operation, and announce success/failure without depending on the icon alone. Verify clipboard support in the target host, including `file://` when local web pages are supported.

### Compact numeric steppers

For values that need repeated fine adjustment, group decrement, value, and increment in one compact property row. Allow direct editing when useful and keep real units such as `px` or `%` with the value. Retain square primitives; use the 24px tool context or approximately 28px regular-control baseline where suitable, without forcing reading/settings fields below their readable size. Do not replace every native number input or slider, add duplicate step buttons, or use circular increment/decrement actions.

- Define range, step, precision, empty-value semantics, and boundary/out-of-range behavior from the product. Disable only the relevant increment/decrement action at a limit; do not silently substitute an arbitrary default.
- Retain native number-input semantics where suitable, direct input and keyboard operation, and accessible names for decrement/increment. The group must not depend on pointer-only mini buttons.
- Reserve enough width for expected digits and units. Incrementing, crossing a digit boundary, focusing, or disabling an action must not resize the group or push adjacent properties. Units are field information, not an extra explanatory paragraph.

### Core control state recipes

Use these states according to each control's actual behavior; not every control needs every state. Compactness does not merge hover, pressed, keyboard focus, selection, or disabled meanings. Focus can coexist with selection or an error.

| Control | At rest | Hover / pressed | Persistent state or error | Keyboard focus | Disabled / loading |
|---|---|---|---|---|---|
| Action button | Its primary, neutral, or destructive recipe; square by default | Derived surface/border; primary uses yellow-derived hover/pressed shades | Pressed styling only for a real toggle, not a completed one-shot action | Visible 2px signal outline | Readable muted control; block unavailable actions; reserve feedback space for real loading |
| Text / number / select | Control fill, visible square boundary and label | No movement or padding change | Read-only stays selectable; invalid has danger boundary plus a necessary nearby error | Signal boundary and visible outline; an error remains identifiable | Readable but unavailable; read-only is not disabled |
| Checkbox / radio / switch | Recognizable native or equivalent control; switch has square track/thumb | Preserve hit area and geometry | Checkmark, radio dot, or thumb position plus signal treatment | Focus remains visible on the actual control | Unavailable without erasing its current value; do not invent a generic loading state |
| Segmented radio | Neutral square track; reserved borders and equal treatment | Control-hover surface; stable text weight | Signal-soft fill, signal border and inset mark for the selected option | Outline on the visible segment | Readable muted option; no selection change through a disabled option |
| Data row | Stable columns and neutral continuous surface | Surface-only hover | Checkbox/marker plus selected surface or inset edge | Visible on the actual interactive cell/control | Disable only unavailable actions; retain row content |
| Menu / checkbox dropdown | Neutral trigger; panel closed | Interactive-row hover without displacement | Open trigger has signal boundary; checked options retain checkmarks and soft fill | Visible trigger/option focus; Escape returns to trigger | Unavailable items cannot execute; do not confuse no results with loading |

Borders, padding, type size/weight, row heights, and feedback slots remain geometrically stable through these states. Verify the combinations actually used in dark/light modes and with reduced motion. Do not invent control behavior to fill the matrix.

## 8. Interaction contract

- Visible 2px signal focus and a sensible Tab sequence are mandatory.
- Icon-only controls need accessible names; relevant tooltips supplement rather than replace those names.
- True tabs support Left/Right and Home/End, with a single tab stop.
- Segmented choices retain native radio arrow navigation and a single Tab entry; checkboxes use Tab/Space. Checkbox dropdowns open with Enter/Space, expose options via Tab and Space, and close on Escape with trigger focus restored or on outside interaction. The expanded panel is out of normal document flow.
- Modals support Escape, initial/trapped focus, background scroll locking, and focus restoration.
- Menus support Escape and outside interaction; disclosure remains keyboard operable.
- Active, hover, selected, disabled, invalid, and loading states are different, even in a small surface.
- Prefer stable widths and local scrolling; avoid layout jumps as a property becomes visible.
- Use 150–200ms state transitions; a necessary, more complex disclosure may use 300ms. Avoid perpetual decorative motion and respect `prefers-reduced-motion`.
- Do not remove permissions, labels, errors, undo/cancel semantics, or real fields to save space.
- State feedback matches actual operation results. Never present an unperformed or failed operation as successful.

## 9. Light mode

| Role | Light value |
|---|---|
| Canvas / surface / panel | `#edf1e4` / `#f8faf3` / `#ffffff` |
| Rail | `#e4e9dc` |
| Control / hover | `#f1f4ea` / `#e7ecdf` |
| Text / muted | `#10150e` / `#5d6659` |
| Hairline | `rgba(16, 21, 14, 0.16)` |

Preserve the base accents; darken accent text by mixing with ink: yellow 34%, green/blue 52%, red 68%. Change the palette only; preserve density, square geometry, and information hierarchy.

An intentionally fixed dark canvas in a host product can be a separate rendering context; do not mistake it for a requirement to ignore light mode on surrounding controls.

## 10. Exclusions

Do not add giant radii, wide decorative gutters, glass blur, floating glow, large decorative headings, repeated card shells, or persistent labels that duplicate the nearby object. Do not split directly readable content or properties into a wall of equally important metric cards.

Reducing wasted space does not remove necessary gaps or readable long-text areas. Floating toolbars may use the named corner variant, and reading/settings regions may use more generous density.

## 11. Ready-to-use implementation brief

> Copy constraint: no decorative English eyebrows, all-caps micro-labels, or slogan subtitles. Do not add explanatory copy by default or explain self-evident buttons and fields. Retain only short hints essential to understanding the task or preventing error/risk, necessary field labels, and accessible names.

> Selection constraint: use themed segmented radios for a few short single-choice options and prefer dropdowns for larger or non-fitting sets. For multiple choice, use horizontal/vertical checkbox groups or a dropdown containing checkboxes. Checking changes state only, not control geometry, table identity, or list position.

> Detail constraint: where the task needs them, use in-place technical-value copying with truthful feedback and compact numeric steppers with explicit units, limits, and stable width. Keep headers visible for genuinely long comparative tables within local scrolling, without adding chrome to small lists; preserve the fidelity of inspection previews. Do not equate data direction with success/failure. Follow the applicable control state recipes in both palette modes. Do not add unnecessary controls or business capabilities just to satisfy these recipes.

> Apply the neuro-mini theme to the target interface without changing its business definition or feature scope. Use the exact near-black neutral palette, signal-yellow active/primary/focus states, green identity/success, cyan information, and red danger. Default to square primitives, thin boundaries, and little shadow. Save space through simpler structure, containers, and repeated labels before reducing text; do not use global scaling. Use 24px tool contexts, approximately 28px regular controls, and readable 36px settings fields according to content. Use system UI text and monospace technical values. Let content determine layout without requiring a toolbar, rail, or inspector. Keep common properties near the affected object and reveal advanced controls on demand. A standalone floating toolbar may use the 9px/5px corner variant; do not extend it to primitives. Preserve necessary labels, permissions, keyboard operation, Escape, focus restoration, truthful states, local scrolling, narrow layouts, and reduced motion. Reuse project components.

## 12. Acceptance checklist

- [ ] Structure and spacing save space without global scaling or unreadable text; the product definition and feature scope remain unchanged.
- [ ] The exact base colors and their semantic roles are preserved.
- [ ] Default primitives are square and containers are economical.
- [ ] The floating-toolbar variant has explicit boundaries and is not extended to primitives.
- [ ] Property tools, ordinary forms, and reading dialogs have appropriate separate density.
- [ ] Active, focus, disabled, invalid, loading, and empty states remain understandable.
- [ ] Names, permissions, errors, and necessary controls were not removed for density.
- [ ] No decorative English headings, slogans, or unnecessary explanations; necessary labels, errors, and risk information remain intact.
- [ ] Single/multiple-choice forms fit the option set and available space; checking preserves control geometry, table columns, and list position.
- [ ] Where copying is offered, its scope, permissions, truthful in-place feedback, stable geometry, and host clipboard behavior are verified.
- [ ] Where numeric steppers are used, units, range, step, precision, empty/boundary behavior, keyboard operation, and width are intentional.
- [ ] Long comparative tables retain visible aligned headers within bounded scrolling; short lists gain no unnecessary chrome.
- [ ] Inspection previews preserve content and aspect ratio; thumbnail cropping is deliberate and the theme does not distort the content.
- [ ] Data-change direction is not confused with success/failure; units and comparison basis are clear.
- [ ] Applicable control states and their combinations follow the recipe matrix without geometry changes and are checked in dark/light modes.
- [ ] Keyboard, modal focus, Escape, scrolling, light mode, and narrow layouts work.
- [ ] Controls with the same semantic role remain consistent across pages.
- [ ] State feedback matches actual operation results.
