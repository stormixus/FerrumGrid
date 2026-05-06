# FerrumGrid

The open-source Navicat alternative. A fast, native PostgreSQL client built with Rust and egui.

<!-- TODO: Add screenshot GIF here -->
<!-- ![FerrumGrid Screenshot](docs/screenshot.png) -->

## Features

- **Query Editor** with syntax highlighting, auto-completion, and multi-tab support
- **Data Grid** with inline editing, sorting, filtering, and CSV/TSV export
- **Schema Browser** with tree navigation for tables, views, functions, and roles
- **ER Diagram** for visualizing table relationships and foreign keys
- **Backup & Restore** via pg_dump/pg_restore integration
- **Vault** for encrypted credential storage
- **Dark & Light themes** with a Supabase-inspired design system
- **macOS native** with traffic lights, Dock menu, and system font support
- **i18n** with English, Korean, and Chinese support

## Install

### Build from source

Prerequisites: [Rust toolchain](https://rustup.rs/) (1.75+)

```bash
git clone https://github.com/stormixus/FerrumGrid.git
cd FerrumGrid
cargo build --release
```

The binary will be at `target/release/ferrumgrid`.

### macOS

```bash
cargo build --release
# Run directly
./target/release/ferrumgrid
```

<!-- TODO: Add Homebrew tap and .dmg download -->

## Development

```bash
cargo build          # Debug build
cargo test           # Run 276+ tests
cargo clippy         # Lint
cargo run            # Run in debug mode
```

## Architecture

```
src/
  app.rs             # Application loop, event handling
  main.rs            # Entry point, window setup
  db/                # PostgreSQL connection, query execution
  state/             # Application state management
  types/             # Data types (CellValue, TableInfo, etc.)
  ui/
    theme.rs         # Design system tokens (see DESIGN.md)
    panels.rs        # Main toolbar and layout
    tree_browser.rs  # Schema tree navigation
    editor.rs        # Query editor with tabs
    grid/            # Data grid (render, selection, editing)
    settings.rs      # Preferences window
    er_diagram.rs    # Entity-relationship diagram
    dialogs.rs       # Connection dialog
    vault.rs         # Credential vault UI
```

## Why FerrumGrid?

| | FerrumGrid | Navicat | DBeaver | pgAdmin |
|---|---|---|---|---|
| Price | Free | $18/mo | Free | Free |
| Speed | Native (Rust) | Native | Java | Web-based |
| UI Quality | Supabase-inspired | Best-in-class | Complex | Dated |
| Open Source | Yes | No | Partial | Yes |

## License

MIT License. See [LICENSE](LICENSE) for details.
