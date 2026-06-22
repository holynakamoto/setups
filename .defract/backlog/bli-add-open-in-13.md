---
id: bli-add-open-in-13
rawText: ''
title: Add keybind to open catalyst URL in default browser
type: task
epic: Hyperlink interaction
module: ui
size: s
labels: []
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:02:12Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:01:25Z
- type: grooming_completed
  timestamp: 2026-06-22T15:02:12Z
  summary: Cleaned title, classified as task under Hyperlink interaction / ui / s; no duplicate found
---

Captured by chat agent during this chat: Builder chose the open-in-browser interaction model for the TUI; ratatui's cell buffer can't render true OSC 8 links reliably, so a keybind is the robust path.

Add an `open` (e.g. `o`) keybinding in the dashboard run loop (dashboard.rs:43-49) that opens the highlighted setup's `catalyst_url` in the default browser, using a crate like `open` or `webbrowser`. Update the footer hint (dashboard.rs:264-274) to advertise it. No-op gracefully when the setup has no URL.
