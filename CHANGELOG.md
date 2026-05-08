# Changelog

All notable changes to FerrumGrid will be documented in this file.

## [0.3.0] - 2026-05-09

### Added

- Built-in SQL backup engine (`BackupFormat::SqlOnly`) — produces a self-contained replayable `.sql` file via `tokio-postgres` without shelling out to `pg_dump`. Streams DDL (CREATE SCHEMA / TABLE / PK / UNIQUE / CHECK / INDEX / FK) plus table data through `COPY ... TO STDOUT`. Supports identity columns, partitioned tables, schema-qualified FK definitions, and topological FK ordering. Output buffered through 64KB `BufWriter`. Partial-file cleanup on error.

### Changed

- Grid module split for maintainability:
  - `info_panel.rs` 1490 → 711 lines (extracted 9 object-info renderers into new `object_info.rs`)
  - `render.rs` 1126 → 630 lines (extracted table layout + cell rendering into new `table_render.rs`)

### Fixed

- 5 clippy warnings (loop counter idiom in command palette, needless borrows in settings panes)

## [0.2.0] - 2026-05-08

### Added

- Command Palette (⌘K) with search, keyboard navigation, and action execution
- Settings window with 10 panes (General, Editor, Data Grid, Connections, Vault, Backup, AI, Diagnostics, Language, Updates) and 46 new configurable fields
- Info panel tabs (Cell, Row, Schema, SQL) with SQL copy button
- Tree panel tabs (Schema, Roles, History, Snippets) with content switching
- Connection mini-status indicator in tree panel header (LED dot + hostname)
- NanumGothicCoding font embedded for Korean language support
- Letter badge icons for tree browser (D/S/T/V/M/ƒ/Q/B) matching mockup design
- Empty grid area click-to-deselect for cell editing
- i18n support for ~160 new UI strings across 7 languages (Korean native translations)

### Changed

- Main toolbar restructured: 64px icon cards replaced with 38px compact tab buttons
- Workspace tabs restyled: top accent bar, rounded corners, bg_darkest fill, view suffix labels
- Titlebar enhanced: connection info display, Dark/Light theme toggle pill, 32px height
- Settings redesigned: sidebar navigation, responsive sizing (min 480x360, max 720x540), dim overlay
- Data grid: 48px row numbers with bg_shell, right-aligned numbers (ACCENT_YELLOW), subtle cell hover, border-subtle separators
- Diagnostics panel: grid-based row layout (timestamp/level/channel/message columns), 160px height
- Info panel default width increased to 280px
- Status bar height reduced to 22px
- Accent color unified to ACCENT_EMERALD across tree browser, grid, editor, and all selection states
- Editable cell colors aligned with mockup (numbers=yellow, JSON=purple, dates=text_secondary, UUID=muted)
- Tree browser icon tinting uses text_secondary for theme compatibility
- Settings row layout uses justify-between (labels left, controls right)
- Metric chip label padding increased by 4px each side

### Fixed

- Table list action icons overlapping (scope_builder double allocation replaced with paint_at)
- Light theme titlebar dark line between titlebar and toolbar
- Cell editing not deactivating when clicking outside grid area
- Command palette overlay covering the palette itself (z-order fix)
- Settings overlay covering the settings window (z-order fix)

### Removed

- ~200 lines of dead code from old toolbar design (settings.rs, icons_svg.rs, theme.rs, state/mod.rs)

## [0.1.4] - 2026-05-08

### Changed

- App icon refined for native macOS Dock appearance: gradient background
  with material depth, inner highlight, radial spotlight, soft edge
  separation (no visible vector outlines)
- Squircle rebuilt as a single continuous path instead of corner cutouts
- 10% canvas padding to match standard macOS icon sizing
- Regenerated `.icns` with updated artwork

## [0.1.3] - 2026-05-07

### Changed

- New app icon: SVG-based minimalist data-storage mark, replacing the
  procedural SDF renderer
- macOS .app bundle now ships with a proper `.icns` icon
  (`assets/AppIcon.icns`, generated from `app-icon-dark.svg` via
  `scripts/generate-icns.sh`)
- Light/dark variants generated from a single source SVG via color swap
  (`#171718 ↔ #ECECEB`, `#12191C → #E2E5E8`); green data-flow accents
  preserved in both themes

## [0.1.2] - 2026-05-07

### Added

- Multi-format export: Export dropdown with CSV, JSON, SQL INSERT options
- Table Designer ALTER DDL: proper column diff engine detects ADD/DROP
  COLUMN, TYPE changes, SET/DROP NOT NULL, SET/DROP DEFAULT
- Data grid Add Row / Delete Row buttons with INSERT/DELETE SQL generation
- Row number gutter column with color coding (red=deleted, teal=inserted)
- Query history side panel: browse and reload past queries from editor
- ER diagram scroll wheel zoom
- ER diagram animated loading spinner
- i18n keys for grid_add_row / grid_delete_row (en/ko)

### Changed

- Cell selection glow strengthened (alpha 34→55, border 1→1.5px teal)
- Double-click-to-edit replaces single-click-to-edit in data grid
- Row hover highlighting with subtle teal tint
- Window/modal shadows enabled for visual depth (dark & light themes)
- FK relationship labels always visible in ER diagram (smaller font in
  dense mode instead of hiding)

## [0.1.1] - 2026-05-06

### Added

- Context-aware info panel for all 12 main views (Connection, Tables, Views,
  Materialized Views, Functions, Roles, Query, Data, Backup, Automation, Model, BI)
- Single-click selection in object lists and tree browser populates the right
  info panel with table/function/role metadata (columns, indexes, FKs)
- Drag & drop a table from the tree browser into the SQL editor — inserts the
  quoted `schema.table` identifier at the cursor
- Icon-only action chips (View Data / Design / Copy SQL / Drop) on the Tables
  / Views / Materialized Views list, with hover tooltips
- Horizontal scroll for object lists when the row exceeds visible width
- Close (×) button on the result panel toolbar with slide-out animation; panel
  slides up smoothly when shown
- New i18n keys for the per-view info strings (en/ko)

### Fixed

- ER diagram cards no longer paint outside the canvas — clip rect applied to
  the per-card painter so cards can't bleed into the workspace tabs above
- Object-list horizontal overflow now produces a scrollbar instead of being
  clipped by the auto-shrunk row width
- Switching to a Query/Connection/object tab no longer surfaces the previous
  Data tab's result rows in the bottom panel
- Object search prioritizes exact-name matches when one exists (`TaxBill` no
  longer also returns `TaxBillItem`)

### Changed

- WorkspaceTab now carries a stable `id` (UUID) for future per-tab state
- Result panel only renders when content (query result, running query, or
  error) is present — no more empty "No result set" panel

### Added

- Query editor with syntax highlighting, multi-tab support, and keyboard shortcuts
- Data grid with inline editing, sorting, filtering, and CSV/TSV export
- Schema browser with tree navigation (tables, views, functions, roles)
- ER diagram for visualizing table relationships
- Backup and restore via pg_dump/pg_restore
- Encrypted credential vault
- Connection dialog with test/save/import
- Dark and light themes (Supabase-inspired design system)
- macOS native integration (traffic lights, Dock menu, borderless titlebar)
- i18n support (English, Korean, Chinese)
- Compact density UI with unified emerald accent palette
- 276+ automated tests
