# Changelog

All notable changes to FerrumGrid will be documented in this file.

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
