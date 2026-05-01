/// Icon constants for FerrumGrid UI.
/// Uses clean Unicode symbols — no emoji, no Nerd Font dependency.
/// Characters chosen for visual clarity at small sizes in monospace context.

// Database objects
pub const DATABASE: &str = "\u{25C6}";       // ◆  filled diamond — strong, compact
pub const SCHEMA: &str = "\u{25A6}";         // ▦  square with crosshatch — layered structure
pub const TABLE: &str = "\u{25A4}";          // ▤  square with horizontal fill — row metaphor
pub const VIEW: &str = "\u{25A1}";           // □  open square — transparent/virtual
pub const MATERIALIZED_VIEW: &str = "\u{25A3}"; // ▣  square with fill — solid view
pub const COLUMN: &str = "\u{2502}";         // │  vertical bar — column metaphor
pub const KEY: &str = "\u{2756}";            // ❖  diamond with cross — primary key marker
pub const INDEX: &str = "\u{2261}";          // ≡  triple bar — indexed layers
pub const FUNCTION: &str = "\u{03BB}";       // λ  lambda — function symbol
pub const TRIGGER: &str = "\u{2609}";        // ☉  sun — trigger/event

// Navigation / tree
pub const FOLDER_OPEN: &str = "\u{25BC}";    // ▼  filled down triangle — expanded
pub const FOLDER_CLOSED: &str = "\u{25BA}";  // ►  filled right triangle — collapsed
pub const CHEVRON_RIGHT: &str = "\u{203A}";  // ›  single right angle — breadcrumb
pub const DOT: &str = "\u{2022}";            // •  bullet

// Connection states
pub const CONNECT: &str = "\u{25CF}";        // ●  filled circle — connected/live
pub const DISCONNECT: &str = "\u{25CB}";     // ○  open circle — disconnected
pub const CONNECTING: &str = "\u{25D4}";     // ◔  circle quarter fill — in-progress

// Actions
pub const EXECUTE: &str = "\u{25B6}";        // ▶  play — run query
pub const PLUS: &str = "+";
pub const CLOSE: &str = "\u{00D7}";          // ×  multiplication sign — close/delete
pub const EXPORT: &str = "\u{2BAB}";         // ⮫  curved arrow right — export
pub const COPY: &str = "\u{2398}";           // ⎘  clipboard — copy
pub const SETTINGS: &str = "\u{2699}";       // ⚙  gear — settings
pub const CANCEL: &str = "\u{25A0}";         // ■  stop — cancel operation

// Status indicators
pub const ERROR: &str = "\u{25BC}";          // ▼  down triangle — error/critical
pub const SUCCESS: &str = "\u{25B2}";        // ▲  up triangle — success/ok
pub const WARNING: &str = "\u{25B6}";        // ▶  right triangle — warning/notice
pub const SPINNER_FRAMES: &[&str] = &[
    "\u{25DC}", "\u{25DD}", "\u{25DE}", "\u{25DF}",
]; // ◜◝◞◟  quarter-circle arcs — spinner animation
pub const NULL_MARKER: &str = "\u{2205}";    // ∅  empty set — NULL value
pub const TRUNCATED: &str = "\u{2026}";      // …  ellipsis — truncated result
pub const LOCK: &str = "\u{25AA}";           // ▪  small square — TLS/locked
pub const UNLOCKED: &str = "\u{25AB}";       // ▫  small open square — no TLS
