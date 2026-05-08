# FerrumGrid Design Standard

Reference: Supabase-inspired DESIGN.md from VoltAgent/awesome-design-md
https://github.com/VoltAgent/awesome-design-md/blob/main/design-md/supabase/DESIGN.md

## Direction

FerrumGrid uses a Supabase-inspired, dark-mode-native developer tool aesthetic: near-black canvases, dense database workflows, thin border-defined depth, and emerald accents used as identity markers. The product should feel like a polished PostgreSQL console, not a marketing page.

## Palette

### Backgrounds (dark / light)

| Token | Dark | Light | Usage |
|-------|------|-------|-------|
| Canvas / `bg_shell` | #171717 | #F1F3F7 | Main surface, all panels |
| Deep / `bg_darkest` | #0F0F0F | #F7F8FB | Code editor, inputs |
| Raised / `bg_medium` | #1F1F1F | #FFFFFF | Toolbars, active tabs |
| `bg_light` | #292929 | #EBEFF5 | Hover states, elevated content |
| `bg_elevated` | #2E2E2E | #FFFFFF | Pressed states, dropdowns |

### Text

| Token | Dark | Light | Usage |
|-------|------|-------|-------|
| `text_primary` | #FAFAFA | #1E222A | Body text, labels |
| `text_secondary` | #B4B4B4 | #525A6A | Descriptions, hints |
| `text_muted` | #898989 | #788192 | Disabled labels, chevrons |
| `text_disabled` | #4D4D4D | #A8B0BE | Fully disabled elements |

### Accent

| Token | Value | Usage |
|-------|-------|-------|
| `ACCENT_EMERALD` | #3ECF8E | Primary brand, active states, buttons |
| `ACCENT_EMERALD_LIGHT` | #5CE6A7 | Hover accent, links |
| `ACCENT_EMERALD_DIM` | #164E37 | Active tab backgrounds, pressed |
| `ACCENT_TEAL` | #00C573 | Tree selection, sort indicators |

### Semantic

| Token | Value | Usage |
|-------|-------|-------|
| `ACCENT_BLUE` | #769CFF | SQL keywords, toolbar icons |
| `ACCENT_GREEN` | #3ECF8E | Success states, connected dot |
| `ACCENT_RED` | #E5484D | Error labels, delete actions |
| `ACCENT_RED_SOFT` | #DC9696 | Error body text (readable) |
| `ACCENT_YELLOW` | #F5CC5D | Warnings, connecting dot, numbers |

### Borders

| Token | Dark | Light | Usage |
|-------|------|-------|-------|
| `border_subtle` | #242424 | #E2E7EE | Panel separators |
| `border_default` | #2E2E2E | #CDD4E0 | Input borders, buttons |
| `border_strong` | #363636 | #B0BAC6 | Hover borders |

## Components

- Primary actions are pill-shaped, dark-filled, and defined by an emerald border.
- Secondary actions use `RADIUS_MD` (4px), neutral dark fill, and gray border.
- Ghost actions stay quiet and rely on hover state instead of permanent borders.
- Cards and panels use dark surfaces, 1px borders, and no decorative shadow.
- Inputs use deep black fills (#0F0F0F), muted placeholder text, and green focus accents.
- Tabs and selected states use emerald as border, underline, or text accent.
- Database tree context menus follow macOS-style native menus.

### Button Specs

| Type | Fill | Stroke | Radius | Text |
|------|------|--------|--------|------|
| Primary | `bg_darkest` | 1px `ACCENT_EMERALD` | pill (255) | White 12.5px |
| Secondary | `bg_medium` | 1px `border_default` | 4px | `text_secondary` 12.5px |
| Ghost | `bg_darkest` | none | 4px | `text_secondary` 12.5px |

### Input Specs

- Height: 28px, background: `input_bg` (#0F0F0F)
- Margin: 8px horizontal, 4px vertical
- Corner radius: 4px

## Typography

| Style | Font | Size | Usage |
|-------|------|------|-------|
| Body | SF Pro | 12px | Default text |
| Monospace | SF Mono | 12px | SQL, code, data |
| Small | SF Pro | 10.5px | Hints, badges |
| Heading | SF Pro | 13.5px | Section titles |

CJK: AppleSDGothicNeo (ko), STHeiti (zh-CN), plus system fallback stack.

## Spacing

| Token | Value |
|-------|-------|
| `SPACE_XS` | 2px |
| `SPACE_SM` | 4px |
| `SPACE_MD` | 8px |
| `SPACE_LG` | 12px |
| `SPACE_XL` | 16px |
| `SPACE_XXL` | 24px |

## Corner Radius

| Token | Value | Usage |
|-------|-------|-------|
| `RADIUS_SM` | 2px | Badges, stripes |
| `RADIUS_MD` | 4px | Buttons, inputs |
| `RADIUS_LG` | 6px | Tabs, toolbar items |

## Guardrails

- Do not return to copper/orange as a primary identity color.
- Do not use large green filled surfaces.
- Do not add decorative shadows; separate surfaces with border contrast.
- Do not use oversized cards or landing-page composition inside the app.
- Keep operational screens dense, scannable, and keyboard-friendly.
