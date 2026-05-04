# FerrumGrid Design Standard

Reference: Supabase-inspired DESIGN.md from VoltAgent/awesome-design-md
https://github.com/VoltAgent/awesome-design-md/blob/main/design-md/supabase/DESIGN.md

## Direction

FerrumGrid uses a Supabase-inspired, dark-mode-native developer tool aesthetic: near-black canvases, dense database workflows, thin border-defined depth, and emerald accents used as identity markers. The product should feel like a polished PostgreSQL console, not a marketing page.

## Palette

- Canvas: `#171717`
- Deep surface: `#0f0f0f`
- Raised surface: `#1f1f1f` to `#292929`
- Borders: `#242424`, `#2e2e2e`, `#363636`
- Primary text: `#fafafa`
- Secondary text: `#b4b4b4`
- Muted text: `#898989`
- Disabled text: `#4d4d4d`
- Brand accent: `#3ecf8e`
- Action/link green: `#00c573`

## Components

- Primary actions are pill-shaped, dark-filled, and defined by an emerald or light border.
- Secondary actions use the standard 6px radius, neutral dark fill, and gray border.
- Ghost actions stay quiet and rely on hover state instead of permanent bright borders.
- Cards and panels use dark surfaces, 1px borders, and no decorative shadow.
- Inputs use deep black fills, muted placeholder text, and green focus accents.
- Tabs and selected states may use emerald only as a border, underline, small dot, or text accent.
- Database tree context menus follow macOS/Navicat-style native menus: light translucent panels, 18px rounded corners, large 44px rows, thin section dividers, gray disabled items, right-aligned shortcuts, and chevrons for nested actions.

## Typography

- Use the system proportional font with regular weight as the default.
- Use medium weight only for buttons, labels, tabs, and navigation.
- Use monospace sparingly for SQL, numeric data, identifiers, and technical labels.
- Avoid heavy bold text; hierarchy comes from size, contrast, and spacing.

## Guardrails

- Do not return to copper/orange as a primary identity color.
- Do not use large green filled surfaces.
- Do not add decorative shadows; separate surfaces with border contrast.
- Do not use oversized cards or landing-page composition inside the app.
- Keep operational screens dense, scannable, and keyboard-friendly.
