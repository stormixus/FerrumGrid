//! Custom designed SVG icons for FerrumGrid.
//! Designed to match Navicat's aesthetic.

pub const CONNECTION: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="connection_g" x1="12" y1="8" x2="54" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#74F2A2"/><stop offset="1" stop-color="#28B86D"/></linearGradient></defs>
    <rect x="10" y="34" width="44" height="20" rx="7" fill="#16231D" stroke="url(#connection_g)" stroke-width="3"/>
    <path d="M23 34V27C23 22.0294 27.0294 18 32 18C36.9706 18 41 22.0294 41 27V34" stroke="url(#connection_g)" stroke-width="5" stroke-linecap="round"/>
    <rect x="27" y="8" width="10" height="22" rx="3" fill="url(#connection_g)"/>
    <circle cx="32" cy="44" r="5" fill="#74F2A2"/>
    <path d="M18 44H25M39 44H46" stroke="#74F2A2" stroke-width="2" stroke-linecap="round" opacity=".65"/>
</svg>"##;

pub const TABLE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="table_g" x1="9" y1="10" x2="55" y2="54" gradientUnits="userSpaceOnUse"><stop stop-color="#F6A15A"/><stop offset="1" stop-color="#B65F24"/></linearGradient></defs>
    <rect x="9" y="11" width="46" height="42" rx="6" fill="#1B1410" stroke="url(#table_g)" stroke-width="3"/>
    <rect x="13" y="15" width="38" height="9" rx="2" fill="url(#table_g)"/>
    <path d="M14 31H50M14 41H50M25 25V51M39 25V51" stroke="#F6A15A" stroke-width="2" stroke-linecap="round" opacity=".58"/>
    <rect x="17" y="31" width="5" height="4" rx="1" fill="#FFD0A6" opacity=".55"/>
    <rect x="29" y="41" width="6" height="4" rx="1" fill="#FFD0A6" opacity=".38"/>
</svg>"##;

pub const VIEW: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="view_g" x1="10" y1="14" x2="54" y2="50" gradientUnits="userSpaceOnUse"><stop stop-color="#8BCBFF"/><stop offset="1" stop-color="#3D8ED6"/></linearGradient></defs>
    <rect x="9" y="12" width="46" height="40" rx="7" fill="#0F1B27" stroke="url(#view_g)" stroke-width="3"/>
    <path d="M14 32C19 23.5 25.5 20 32 20C38.5 20 45 23.5 50 32C45 40.5 38.5 44 32 44C25.5 44 19 40.5 14 32Z" fill="#142A3C" stroke="url(#view_g)" stroke-width="3" stroke-linejoin="round"/>
    <circle cx="32" cy="32" r="8" fill="#8BCBFF"/>
    <circle cx="32" cy="32" r="4" fill="#0F1B27"/>
</svg>"##;

pub const MATERIALIZED_VIEW: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="mat_g" x1="9" y1="9" x2="55" y2="55" gradientUnits="userSpaceOnUse"><stop stop-color="#5BF0E2"/><stop offset="1" stop-color="#1D9CBA"/></linearGradient></defs>
    <rect x="12" y="8" width="40" height="40" rx="7" fill="#102326" stroke="url(#mat_g)" stroke-width="3"/>
    <rect x="8" y="16" width="40" height="40" rx="7" fill="#122A2E" stroke="url(#mat_g)" stroke-width="3" opacity=".86"/>
    <path d="M15 36C19 29 24 26 29 26C34 26 39 29 43 36C39 43 34 46 29 46C24 46 19 43 15 36Z" stroke="#9CFBF2" stroke-width="3" stroke-linejoin="round"/>
    <circle cx="29" cy="36" r="5" fill="#9CFBF2"/>
</svg>"##;

pub const FUNCTION: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="fn_g" x1="14" y1="10" x2="50" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#FFE88A"/><stop offset="1" stop-color="#D9A51E"/></linearGradient></defs>
    <path d="M18 51C23 51 23 43 24 35L26 21C27 13 31 9 39 9" stroke="url(#fn_g)" stroke-width="6" stroke-linecap="round"/>
    <path d="M15 28H41" stroke="#FFE88A" stroke-width="4" stroke-linecap="round" opacity=".9"/>
    <path d="M36 23L49 36L36 49" stroke="url(#fn_g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
    <circle cx="39" cy="9" r="4" fill="#FFF2A6"/>
    <circle cx="18" cy="51" r="4" fill="#D9A51E"/>
</svg>"##;

pub const USER: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="24" r="12" fill="#E69850"/>
    <path d="M12 52C12 43.1634 19.1634 36 28 36H36C44.8366 36 52 43.1634 52 52V56H12V52Z" fill="#E69850"/>
</svg>"##;

pub const QUERY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="query_g" x1="12" y1="8" x2="52" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#9AD7FF"/><stop offset="1" stop-color="#3279D6"/></linearGradient></defs>
    <rect x="13" y="8" width="38" height="48" rx="7" fill="#102033" stroke="url(#query_g)" stroke-width="3"/>
    <path d="M22 22H42M22 31H35M22 40H39" stroke="#9AD7FF" stroke-width="3" stroke-linecap="round" opacity=".9"/>
    <path d="M39 29L47 34L39 39V29Z" fill="#9AD7FF"/>
</svg>"##;

pub const BACKUP: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="backup_g" x1="10" y1="10" x2="54" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#AAB4C6"/><stop offset="1" stop-color="#626E84"/></linearGradient></defs>
    <path d="M50 32C50 42 42 50 32 50C22 50 14 42 14 32C14 22 22 14 32 14C38 14 43 17 46 21" stroke="url(#backup_g)" stroke-width="5" stroke-linecap="round"/>
    <path d="M45 12V23H56" stroke="url(#backup_g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
    <rect x="22" y="28" width="20" height="14" rx="4" fill="#10151D" stroke="#AAB4C6" stroke-width="3"/>
    <path d="M28 28V24C28 21.7909 29.7909 20 32 20C34.2091 20 36 21.7909 36 24V28" stroke="#AAB4C6" stroke-width="3" stroke-linecap="round"/>
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
    <defs><linearGradient id="db_g" x1="8" y1="8" x2="56" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#7FC4FF"/><stop offset="1" stop-color="#2F7FD0"/></linearGradient></defs>
    <path d="M10 19C10 12.3726 19.8497 7 32 7C44.1503 7 54 12.3726 54 19V45C54 51.6274 44.1503 57 32 57C19.8497 57 10 51.6274 10 45V19Z" fill="#0F1C2A" stroke="url(#db_g)" stroke-width="3"/>
    <ellipse cx="32" cy="19" rx="22" ry="12" fill="#132A40" stroke="url(#db_g)" stroke-width="3"/>
    <path d="M54 32C54 38.6274 44.1503 44 32 44C19.8497 44 10 38.6274 10 32" stroke="#7FC4FF" stroke-width="3" opacity=".7"/>
    <path d="M54 44C54 50.6274 44.1503 56 32 56C19.8497 56 10 50.6274 10 44" stroke="#7FC4FF" stroke-width="3" opacity=".55"/>
</svg>"##;

pub const SCHEMA: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="schema_g" x1="10" y1="10" x2="54" y2="54" gradientUnits="userSpaceOnUse"><stop stop-color="#63F2D8"/><stop offset="1" stop-color="#24A89D"/></linearGradient></defs>
    <rect x="10" y="10" width="44" height="44" rx="8" fill="#102423" stroke="url(#schema_g)" stroke-width="3"/>
    <path d="M20 22H44M20 32H44M20 42H44M22 20V44M32 20V44M42 20V44" stroke="#63F2D8" stroke-width="2" stroke-linecap="round" opacity=".62"/>
    <rect x="18" y="18" width="28" height="28" rx="4" stroke="#A5FFF0" stroke-width="2" opacity=".42"/>
</svg>"##;

pub const COLUMN: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="column_g" x1="20" y1="8" x2="44" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#C3CAD8"/><stop offset="1" stop-color="#6C7588"/></linearGradient></defs>
    <rect x="20" y="8" width="24" height="48" rx="5" fill="#141922" stroke="url(#column_g)" stroke-width="3"/>
    <path d="M25 21H39M25 32H39M25 43H39" stroke="#C3CAD8" stroke-width="3" stroke-linecap="round" opacity=".58"/>
</svg>"##;

pub const INDEX: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="index_g" x1="12" y1="10" x2="52" y2="54" gradientUnits="userSpaceOnUse"><stop stop-color="#9AD7FF"/><stop offset="1" stop-color="#3279D6"/></linearGradient></defs>
    <rect x="13" y="12" width="38" height="40" rx="6" fill="#102033" stroke="url(#index_g)" stroke-width="3"/>
    <path d="M22 22H43M22 32H38M22 42H45" stroke="#9AD7FF" stroke-width="4" stroke-linecap="round"/>
    <circle cx="18" cy="22" r="3" fill="#9AD7FF"/>
    <circle cx="18" cy="32" r="3" fill="#9AD7FF" opacity=".75"/>
    <circle cx="18" cy="42" r="3" fill="#9AD7FF" opacity=".55"/>
</svg>"##;

pub const KEY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="key_g" x1="10" y1="10" x2="54" y2="54" gradientUnits="userSpaceOnUse"><stop stop-color="#FFF09A"/><stop offset="1" stop-color="#D29A18"/></linearGradient></defs>
    <circle cx="24" cy="24" r="12" fill="#16140B" stroke="url(#key_g)" stroke-width="5"/>
    <circle cx="24" cy="24" r="4" fill="#FFF09A"/>
    <path d="M33 33L52 52M43 43L48 38M47 47L52 42" stroke="url(#key_g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const UNIQUE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="unique_g" x1="10" y1="10" x2="54" y2="54" gradientUnits="userSpaceOnUse"><stop stop-color="#FFF09A"/><stop offset="1" stop-color="#E69850"/></linearGradient></defs>
    <path d="M18 30V23C18 15.268 24.268 9 32 9C39.732 9 46 15.268 46 23V30" stroke="url(#unique_g)" stroke-width="5" stroke-linecap="round"/>
    <rect x="13" y="28" width="38" height="27" rx="7" fill="#1B160E" stroke="url(#unique_g)" stroke-width="4"/>
    <circle cx="32" cy="41" r="5" fill="#FFF09A"/>
    <path d="M32 45V50" stroke="#FFF09A" stroke-width="4" stroke-linecap="round"/>
</svg>"##;

pub const RULE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="rule_g" x1="13" y1="8" x2="51" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#C3CAD8"/><stop offset="1" stop-color="#7E8AA0"/></linearGradient></defs>
    <rect x="14" y="8" width="36" height="48" rx="6" fill="#141922" stroke="url(#rule_g)" stroke-width="3"/>
    <path d="M23 22H41M23 32H36M23 42H40" stroke="#C3CAD8" stroke-width="3" stroke-linecap="round"/>
    <path d="M19 16L25 10M45 54L51 48" stroke="#7E8AA0" stroke-width="3" stroke-linecap="round"/>
</svg>"##;

pub const TRIGGER: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="trigger_g" x1="13" y1="8" x2="51" y2="56" gradientUnits="userSpaceOnUse"><stop stop-color="#5BF0E2"/><stop offset="1" stop-color="#24A89D"/></linearGradient></defs>
    <path d="M32 7L19 32H31L25 57L45 26H33L32 7Z" fill="#102423" stroke="url(#trigger_g)" stroke-width="4" stroke-linejoin="round"/>
    <path d="M33 14L25 29H35L31 44L41 28H32L33 14Z" fill="#9CFBF2" opacity=".72"/>
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

pub const CHEVRON_LEFT: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M40 16L24 32L40 48" stroke="currentColor" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub const CHEVRON_DOUBLE_LEFT: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M35 16L19 32L35 48" stroke="currentColor" stroke-width="7" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M49 16L33 32L49 48" stroke="currentColor" stroke-width="7" stroke-linecap="round" stroke-linejoin="round" opacity=".72"/>
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

pub const CALENDAR: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" color="#2DBDFF" xmlns="http://www.w3.org/2000/svg">
    <rect x="10" y="14" width="44" height="40" rx="7" fill="#0D1820" stroke="currentColor" stroke-width="4"/>
    <path d="M10 26H54" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
    <path d="M22 8V18M42 8V18" stroke="currentColor" stroke-width="5" stroke-linecap="round"/>
    <path d="M21 36H25M31 36H35M41 36H45M21 45H25M31 45H35" stroke="currentColor" stroke-width="4" stroke-linecap="round" opacity=".82"/>
</svg>"##;

pub const CLOCK: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" color="#2DBDFF" xmlns="http://www.w3.org/2000/svg">
    <circle cx="32" cy="32" r="23" fill="#0D1820" stroke="currentColor" stroke-width="4"/>
    <path d="M32 18V33L43 39" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
    <circle cx="32" cy="32" r="3" fill="currentColor"/>
</svg>"##;

pub const PLUS: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M32 12V52M12 32H52" stroke="#4EBE64" stroke-width="8" stroke-linecap="round"/>
</svg>"##;

pub const COPY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" color="#2DBDFF" xmlns="http://www.w3.org/2000/svg">
    <rect x="22" y="18" width="30" height="34" rx="5" fill="#0D1820" stroke="currentColor" stroke-width="4"/>
    <path d="M14 42V14C14 11.7909 15.7909 10 18 10H42" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" opacity=".72"/>
    <path d="M30 30H44M30 39H42" stroke="currentColor" stroke-width="3" stroke-linecap="round" opacity=".82"/>
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

pub const SORT: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M22 12V52M22 12L14 20M22 12L30 20" stroke="currentColor" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M42 52V12M42 52L34 44M42 52L50 44" stroke="currentColor" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" opacity=".72"/>
</svg>"##;

pub const SORT_ASC: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M21 52V14M21 14L12 23M21 14L30 23" stroke="currentColor" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M38 22H52M38 32H48M38 42H44" stroke="currentColor" stroke-width="5" stroke-linecap="round"/>
</svg>"##;

pub const SORT_DESC: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M21 12V50M21 50L12 41M21 50L30 41" stroke="currentColor" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M38 22H44M38 32H48M38 42H52" stroke="currentColor" stroke-width="5" stroke-linecap="round"/>
</svg>"##;

pub const CANCEL: &str = CLOSE;
pub const EXECUTE: &str = QUERY;

pub const EDIT: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M44 8L56 20L24 52L8 56L12 40L44 8Z" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
    <path d="M40 12L52 24" stroke="currentColor" stroke-width="5" stroke-linecap="round"/>
</svg>"##;

pub const TRASH: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M14 18H50" stroke="currentColor" stroke-width="5" stroke-linecap="round"/>
    <path d="M24 18V12C24 10.8954 24.8954 10 26 10H38C39.1046 10 40 10.8954 40 12V18" stroke="currentColor" stroke-width="5" stroke-linecap="round"/>
    <path d="M18 18V52C18 53.1046 18.8954 54 20 54H44C45.1046 54 46 53.1046 46 52V18" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M28 28V44M36 28V44" stroke="currentColor" stroke-width="4" stroke-linecap="round" opacity=".7"/>
</svg>"##;

pub const CODE: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M22 18L8 32L22 46" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M42 18L56 32L42 46" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M36 12L28 52" stroke="currentColor" stroke-width="4" stroke-linecap="round" opacity=".75"/>
</svg>"##;

pub const PLAY: &str = r##"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M16 12L52 32L16 52V12Z" fill="currentColor" stroke="currentColor" stroke-width="4" stroke-linejoin="round"/>
</svg>"##;
