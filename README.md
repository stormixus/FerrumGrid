<div align="center">

<img src="assets/app-icon.png#gh-dark-mode-only" width="128" alt="FerrumGrid"/>
<img src="assets/app-icon-light.png#gh-light-mode-only" width="128" alt="FerrumGrid"/>

# FerrumGrid

**A native PostgreSQL desktop client, forged in Rust.**
Fast like the metal it's named after. Sexy like the tools you wish you had. Built with a stunning grid-structured interface.

[![Release](https://img.shields.io/github/v/release/stormixus/FerrumGrid?style=for-the-badge&color=2ecba7&labelColor=171718)](https://github.com/stormixus/FerrumGrid/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge&labelColor=171718)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust&labelColor=171718)](https://www.rust-lang.org/)
[![Platforms](https://img.shields.io/badge/macOS%20·%20Linux%20·%20Windows-supported-8b5cf6?style=for-the-badge&labelColor=171718)](#download)

[**Download**](#download) · [**Quick Start**](#quick-start) · [**Why FerrumGrid?**](#why-ferrumgrid) · [**Roadmap**](#roadmap)

</div>

---

## Why FerrumGrid?

PostgreSQL deserves a client that doesn't feel like a 2008 Java applet or a wrapped Chrome tab. FerrumGrid is built from scratch in Rust with a Supabase-inspired design language — every pixel, every keystroke, every query is tuned for the developers who actually live in the database.

- ⚡ **Native, not wrapped** — no Electron, no JVM, no web view. A single statically-linked Rust binary.
- 🎨 **Designed, not assembled** — coherent dark/light themes, typography that actually breathes, an icon set drawn instead of stolen.
- 🔐 **Encrypted vault** — credentials sealed with Argon2id + ChaCha20-Poly1305. Your `.env` files thank you.
- 🧠 **Built for keyboard wizards** — Command Palette (⌘K), multi-tab editor, instant tree navigation.

## Features

<table>
<tr>
<td width="50%">

**Query & Edit**
- SQL editor with syntax highlighting, multi-tab, history
- Inline cell editing with type-aware coloring (numbers, JSON, UUIDs, dates)
- Add Row / Delete Row with auto-generated INSERT / DELETE
- Multi-format export: CSV, JSON, SQL INSERT

</td>
<td width="50%">

**Explore**
- Schema browser: tables, views, materialized views, functions, roles
- Context-aware info panel for every object type
- Drag tables from the tree directly into the editor
- ER diagram with live FK relationships and zoom-to-fit

</td>
</tr>
<tr>
<td width="50%">

**Operate**
- Built-in **SQL backup engine** — no `pg_dump` dependency
- Streams DDL + `COPY ... TO STDOUT` for partitioned tables, identity columns, schema-scoped FKs
- Table Designer with proper ALTER diff (ADD/DROP COLUMN, TYPE, NOT NULL, DEFAULT)
- Encrypted credential Vault (Argon2id + ChaCha20-Poly1305)

</td>
<td width="50%">

**Polish**
- Command Palette (⌘K) — every action one search away
- Dark / Light themes with unified emerald accent
- 7-language i18n (English, Korean, Chinese, more)
- Native macOS chrome (traffic lights, Dock menu, system fonts)

</td>
</tr>
</table>

## Download

Latest release: **[v0.3.2](https://github.com/stormixus/FerrumGrid/releases/latest)**

| Platform | Installer | Archive |
|---|---|---|
| 🍎 **macOS** Apple Silicon | [.dmg](https://github.com/stormixus/FerrumGrid/releases/latest/download/FerrumGrid-0.3.2-aarch64-apple-darwin.dmg) | [.tar.xz](https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-aarch64-apple-darwin.tar.xz) |
| 🍎 **macOS** Intel | [.dmg](https://github.com/stormixus/FerrumGrid/releases/latest/download/FerrumGrid-0.3.2-x86_64-apple-darwin.dmg) | [.tar.xz](https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-x86_64-apple-darwin.tar.xz) |
| 🐧 **Linux** x86_64 | — | [.tar.xz](https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-x86_64-unknown-linux-gnu.tar.xz) |
| 🪟 **Windows** x64 | [.msi](https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-x86_64-pc-windows-msvc.msi) | [.zip](https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-x86_64-pc-windows-msvc.zip) |

Or, one-shot install via shell:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-installer.sh | sh
```

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/stormixus/FerrumGrid/releases/latest/download/ferrumgrid-installer.ps1 | iex"
```

> **ARM64 Linux is not supported** in prebuilt releases — build from source via `cargo build --release` on the target machine.

## Quick Start

```bash
git clone https://github.com/stormixus/FerrumGrid.git
cd FerrumGrid
cargo run --release
```

**Linux build deps** (Debian / Ubuntu):

```bash
sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
                 libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

Then connect to a database — FerrumGrid will remember it (encrypted) for next time.

## Stack

Built on the shoulders of giants:

- **[egui](https://github.com/emilk/egui)** — immediate-mode GUI in pure Rust
- **[tokio-postgres](https://github.com/sfackler/rust-postgres)** — async PostgreSQL driver
- **[tokio](https://tokio.rs/)** — async runtime
- **[ChaCha20-Poly1305](https://github.com/RustCrypto/AEADs)** + **[Argon2](https://github.com/RustCrypto/password-hashes)** — vault crypto
- **No Electron. No JVM. No web view.** Just a single statically-linked native binary.

## How does it compare?

|  | **FerrumGrid** | DBeaver | pgAdmin | TablePlus |
|---|---|---|---|---|
| Engine | Native Rust | JVM | Web (Python) | Native (closed) |
| Open source | ✅ MIT | Partial | ✅ | ❌ |
| Price | Free | Free | Free | Paid tiers |
| PostgreSQL-first | ✅ | Multi-DB | ✅ | Multi-DB |

## Roadmap

> _Tentative — these are ideas under consideration, not commitments. See [CHANGELOG.md](CHANGELOG.md) for what actually shipped._

- AI Assist: natural-language → SQL with schema context
- Saved snippets library + cross-connection sharing
- Plugin API for custom export formats and view renderers
- Homebrew tap (`brew install --cask ferrumgrid`)
- Linux ARM64 prebuilt artifacts (pending GTK3 cross-compile story)

## Development

```bash
cargo build          # Debug build
cargo test           # Run 276+ tests
cargo clippy         # Lint
cargo run            # Run in debug mode
```

Project layout:

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

Design system tokens are defined in [DESIGN.md](DESIGN.md).

## Contributing

PRs welcome. Open an issue first for non-trivial changes.
The codebase keeps a tight diff-discipline — small, atomic commits with `cargo test` + `cargo clippy` clean.

## License

[MIT](LICENSE) — © 2026 Stormix

<div align="center">

**Built with 🦀 in Seoul.**
If FerrumGrid saved you from another DBeaver crash, [⭐ star the repo](https://github.com/stormixus/FerrumGrid).

</div>
