---
id: bli-redesign-the-ratatui-11
rawText: ''
title: Polish dashboard layout and visual hierarchy
type: improvement
epic: TUI redesign
module: ui
size: s
labels: []
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:01:31Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:00:36Z
- type: grooming_completed
  timestamp: 2026-06-22T15:01:31Z
  summary: Cleaned title, typed as improvement, reused TUI redesign/ui taxonomy from corpus, sized s for bounded dashboard.rs polish work
---

Captured by chat agent during this chat: The builder explicitly asked to make the TUI more visually appealing; this is the presentation-layer work, separable from the data plumbing.

Rework src/ui/dashboard.rs layout and styling: tighter header/footer, clearer column emphasis, better use of color and grade/score highlighting, and a more readable detail panel. Keep the four-pane layout (render at dashboard.rs:73-88) but polish spacing, borders, and visual hierarchy so the scan reads at a glance.
