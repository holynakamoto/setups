# Design System

## Overview

This is a Rust terminal UI application built with [ratatui](https://ratatui.rs/) and [crossterm](https://github.com/crossterm-rs/crossterm). There are no CSS files, web styling frameworks, or design token files. All visual styling is defined in `src/ui/dashboard.rs` using ratatui's `Style`, `Color`, and `Modifier` types, which map to ANSI terminal colors. Exact rendered values depend on the terminal emulator's color theme.

## Colors

### Semantic Colors

| Role | ratatui Token | ANSI Approx |
|------|--------------|-------------|
| Accent / brand | `Color::Cyan` | ANSI 6 (~`#06989A`) |
| Positive / bullish | `Color::Green` | ANSI 2 (~`#4E9A06`) |
| Positive / high-grade | `Color::LightGreen` | ANSI 10 (~`#8AE234`) |
| Warning / moderate | `Color::Yellow` | ANSI 3 (~`#C4A000`) |
| Negative / bearish | `Color::Red` | ANSI 1 (~`#CC0000`) |
| Table header text | `Color::White` | ANSI 7 (~`#D3D7CF`) |
| Row highlight background | `Color::DarkGray` | ANSI 8 (~`#555753`) |

### Color Usage by Context

| Context | Color |
|---------|-------|
| App title "SETUPS" | `Color::Cyan` + `Modifier::BOLD` |
| Setup count in header | `Color::Yellow` |
| Ticker symbol cells | `Color::Cyan` + `Modifier::BOLD` |
| Positive gap % | `Color::Green` |
| Negative gap % | `Color::Red` |
| High RVOL (≥ 3.0×) | `Color::Yellow` |
| Score 90–100 (A grade) | `Color::LightGreen` + `Modifier::BOLD` |
| Score 70–89 (B grade) | `Color::Green` |
| Score 50–69 (C grade) | `Color::Yellow` |
| Score < 50 (D/F grade) | `Color::Red` |
| Keybinding labels in footer | `Color::Yellow` |
| Selected row background | `Color::DarkGray` |

## Typography

### Text Modifiers

| Modifier | Usage |
|----------|-------|
| `Modifier::BOLD` | App title, ticker symbols, table header row, selected row, A-grade scores |

## Layout

### Vertical Layout Regions

| Region | Height | Content |
|--------|--------|---------|
| Header | 3 lines | App title + setup count |
| Table | Min 10 lines | Sortable setup list |
| Detail | 10 lines | Selected setup detail |
| Footer | 2 lines | Keybinding hints |

### Table Column Widths

| Column | Width |
|--------|-------|
| Symbol | 8 chars |
| Price | 9 chars |
| Gap % | 8 chars |
| RVOL | 7 chars |
| Float | 8 chars |
| Short % | 9 chars |
| Catalyst | min 16 chars (expands) |
| Score | 7 chars |
| Grade | 6 chars |

## Components

### Organization

Single UI component: `src/ui/dashboard.rs` (`Dashboard` struct). All rendering is split across four private methods: `render_header`, `render_table`, `render_detail`, `render_footer`.

### Component Count

1 UI component file.

## Conventions

### Styling Approach

- Framework: ratatui TUI (terminal, not web)
- File pattern: single co-located file (`src/ui/dashboard.rs`)
- Colors are ratatui `Color` enum variants (ANSI terminal palette)
- Score-to-color mapping is encoded in a standalone `score_color(f64) -> Style` function

### Score Grading

```
90–100 → LightGreen + BOLD  (grade "A")
70–89  → Green              (grade "B")
50–69  → Yellow             (grade "C")
<50    → Red                (grade "D" or "F")
```
