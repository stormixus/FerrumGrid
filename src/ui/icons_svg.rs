/// Custom designed SVG icons for FerrumGrid.
/// Designed to match Navicat's aesthetic.

pub const CONNECTION: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M12 40C12 37.7909 13.7909 36 16 36H24V32H16C11.5817 32 8 35.5817 8 40V48C8 52.4183 11.5817 56 16 56H48C52.4183 56 56 52.4183 56 48V40C56 35.5817 52.4183 32 48 32H40V36H48C50.2091 36 52 37.7909 52 40V48C52 50.2091 50.2091 52 48 52H16C13.7909 52 12 50.2091 12 48V40Z" fill="#4EBE64"/>
    <rect x="26" y="8" width="12" height="32" rx="2" fill="#4EBE64"/>
    <path d="M22 14H42V22C42 27.5228 37.5228 32 32 32V32C26.4772 32 22 27.5228 22 22V14Z" fill="#4EBE64" fill-opacity="0.3"/>
    <circle cx="32" cy="44" r="6" fill="#4EBE64"/>
</svg>"##;

pub const TABLE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="8" y="12" width="48" height="40" rx="4" fill="#CC7832"/>
    <rect x="12" y="16" width="40" height="8" rx="1" fill="white" fill-opacity="0.2"/>
    <rect x="12" y="28" width="12" height="6" rx="1" fill="white" fill-opacity="0.1"/>
    <rect x="26" y="28" width="26" height="6" rx="1" fill="white" fill-opacity="0.1"/>
    <rect x="12" y="38" width="12" height="6" rx="1" fill="white" fill-opacity="0.1"/>
    <rect x="26" y="38" width="26" height="6" rx="1" fill="white" fill-opacity="0.1"/>
</svg>"##;

pub const VIEW: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="8" y="12" width="48" height="40" rx="4" stroke="#569CD6" stroke-width="4"/>
    <circle cx="32" cy="32" r="10" stroke="#569CD6" stroke-width="3"/>
    <circle cx="32" cy="32" r="4" fill="#569CD6"/>
</svg>"##;

pub const MATERIALIZED_VIEW: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="8" y="12" width="48" height="40" rx="4" fill="#34BEAB"/>
    <circle cx="32" cy="32" r="10" stroke="white" stroke-width="3"/>
    <circle cx="32" cy="32" r="4" fill="white"/>
</svg>"##;

pub const FUNCTION: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M20 48V42C20 38.6863 22.6863 36 26 36H38C41.3137 36 44 33.3137 44 30V16" stroke="#DCC950" stroke-width="6" stroke-linecap="round"/>
    <circle cx="20" cy="48" r="6" fill="#DCC950"/>
    <circle cx="44" cy="16" r="6" fill="#DCC950"/>
    <path d="M30 26L38 26" stroke="#DCC950" stroke-width="4" stroke-linecap="round"/>
</svg>"##;

pub const USER: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="24" r="12" fill="#E69850"/>
    <path d="M12 52C12 43.1634 19.1634 36 28 36H36C44.8366 36 52 43.1634 52 52V56H12V52Z" fill="#E69850"/>
</svg>"##;

pub const QUERY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="12" y="8" width="40" height="48" rx="4" fill="#569CD6"/>
    <path d="M40 24L28 32L40 40V24Z" fill="white"/>
    <rect x="20" y="16" width="24" height="3" rx="1.5" fill="white" fill-opacity="0.3"/>
</svg>"##;

pub const BACKUP: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M32 12C20.9543 12 12 20.9543 12 32C12 43.0457 20.9543 52 32 52C43.0457 52 52 43.0457 52 32" stroke="#646A7A" stroke-width="6" stroke-linecap="round"/>
    <path d="M44 20L52 32L60 20" fill="#646A7A"/>
    <rect x="24" y="28" width="16" height="12" rx="2" fill="#646A7A"/>
</svg>"##;

pub const AUTOMATION: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="20" y="12" width="24" height="24" rx="4" fill="#34BEAB"/>
    <rect x="16" y="40" width="32" height="12" rx="2" fill="#34BEAB" fill-opacity="0.5"/>
    <circle cx="26" cy="20" r="3" fill="white"/>
    <circle cx="38" cy="20" r="3" fill="white"/>
</svg>"##;

pub const MODEL: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="10" y="10" width="16" height="12" rx="2" fill="#4EBE64"/>
    <rect x="38" y="10" width="16" height="12" rx="2" fill="#4EBE64"/>
    <rect x="24" y="42" width="16" height="12" rx="2" fill="#4EBE64"/>
    <path d="M18 22V32H32V42" stroke="#4EBE64" stroke-width="3"/>
    <path d="M46 22V32H32" stroke="#4EBE64" stroke-width="3"/>
</svg>"##;

pub const DATABASE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M32 8C16 8 8 14 8 20V44C8 50 16 56 32 56C48 56 56 50 56 44V20C56 14 48 8 32 8Z" fill="#569CD6" fill-opacity="0.2" stroke="#569CD6" stroke-width="4"/>
    <path d="M56 20C56 26 48 32 32 32C16 32 8 26 8 20" stroke="#569CD6" stroke-width="4"/>
    <path d="M56 32C56 38 48 44 32 44C16 44 8 38 8 32" stroke="#569CD6" stroke-width="4"/>
</svg>"##;

pub const SCHEMA: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="12" y="12" width="40" height="40" rx="4" stroke="#34BEAB" stroke-width="4"/>
    <path d="M12 28H52M12 40H52M28 12V52M40 12V52" stroke="#34BEAB" stroke-width="2" stroke-opacity="0.5"/>
</svg>"##;

pub const COLUMN: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="24" y="8" width="16" height="48" rx="2" fill="#646A7A"/>
    <path d="M24 24H40M24 40H40" stroke="white" stroke-width="2" stroke-opacity="0.3"/>
</svg>"##;

pub const KEY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="24" cy="24" r="12" stroke="#DCC950" stroke-width="6"/>
    <path d="M32 32L48 48M42 42L46 38M45 45L49 41" stroke="#DCC950" stroke-width="6" stroke-linecap="round"/>
</svg>"##;

pub const CLOSE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M16 16L48 48M48 16L16 48" stroke="#D24646" stroke-width="8" stroke-linecap="round"/>
</svg>"##;

pub const REFRESH: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M52 32C52 43.0457 43.0457 52 32 52C20.9543 52 12 43.0457 12 32C12 20.9543 20.9543 12 32 12C38.085 12 43.5413 14.71 47.1989 19" stroke="#34BEAB" stroke-width="6" stroke-linecap="round"/>
    <path d="M40 20H50V10" stroke="#34BEAB" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const INFO: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="32" r="24" stroke="#569CD6" stroke-width="4"/>
    <rect x="30" y="26" width="4" height="18" rx="2" fill="#569CD6"/>
    <circle cx="32" cy="18" r="3" fill="#569CD6"/>
</svg>"##;

pub const BI: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="12" y="40" width="8" height="12" rx="1" fill="#D24646"/>
    <rect x="28" y="24" width="8" height="28" rx="1" fill="#D24646"/>
    <rect x="44" y="12" width="8" height="40" rx="1" fill="#D24646"/>
</svg>"##;

pub const CHEVRON_RIGHT: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M24 16L40 32L24 48" stroke="currentColor" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const CHEVRON_DOWN: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M16 24L32 40L48 24" stroke="currentColor" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const SUCCESS: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="32" r="28" fill="#4EBE64" fill-opacity="0.2"/>
    <path d="M20 32L28 40L44 24" stroke="#4EBE64" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const ERROR: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="32" r="28" fill="#D24646" fill-opacity="0.2"/>
    <path d="M22 22L42 42M42 22L22 42" stroke="#D24646" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const CONNECT: &str = CONNECTION;

pub const PLUS: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M32 12V52M12 32H52" stroke="#4EBE64" stroke-width="8" stroke-linecap="round"/>
</svg>"##;

pub const COPY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="20" y="20" width="32" height="32" rx="4" stroke="#569CD6" stroke-width="4"/>
    <path d="M12 12H44V20H12V44H12V12Z" stroke="#569CD6" stroke-width="4" stroke-opacity="0.5"/>
</svg>"##;

pub const EXPORT: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M32 12V40M32 40L20 28M32 40L44 28" stroke="#34BEAB" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
    <rect x="12" y="48" width="40" height="4" rx="2" fill="#34BEAB"/>
</svg>"##;

pub const TRUNCATED: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="16" cy="32" r="4" fill="currentColor"/>
    <circle cx="32" cy="32" r="4" fill="currentColor"/>
    <circle cx="48" cy="32" r="4" fill="currentColor"/>
</svg>"##;

pub const NULL_MARKER: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="32" r="20" stroke="currentColor" stroke-width="4" stroke-dasharray="8 4"/>
    <path d="M20 20L44 44" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
</svg>"##;

pub const CANCEL: &str = CLOSE;
pub const EXECUTE: &str = QUERY;
