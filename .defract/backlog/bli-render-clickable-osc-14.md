---
id: bli-render-clickable-osc-14
rawText: ''
title: Add OSC 8 hyperlink wrapping to news cells in plain-text output
type: task
epic: Hyperlink interaction
module: cli
size: s
labels:
- news
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:02:24Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:01:34Z
- type: grooming_completed
  timestamp: 2026-06-22T15:02:24Z
  summary: Cleaned title to verb-first, confirmed Hyperlink interaction / cli, sized s, applied news label
---

Captured by chat agent during this chat: The `--plain` path can support genuinely clickable links cheaply, which is the original ask; pairs with the TUI keybind to cover both output modes.

In print_table (src/main.rs:198-244), wrap the news cell in an OSC 8 hyperlink escape (`\e]8;;URL\e\\text\e]8;;\e\\`) when a catalyst_url is present, so the headline is clickable in OSC-8-capable terminals (iTerm2, kitty, modern Terminal). Fall back to plain text when no URL.
