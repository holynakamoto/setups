---
id: bli-thread-finnhub-news-10
rawText: ''
title: Thread Finnhub article URL into Setup
type: task
epic: Hyperlink interaction
module: setups
size: s
labels:
- news
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:01:21Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:00:36Z
- type: grooming_completed
  timestamp: 2026-06-22T15:01:21Z
  summary: Cleaned title to "Thread Finnhub article URL into Setup", filed under Hyperlink interaction epic / setups module, sized s, labelled news
---

Captured by chat agent during this chat: Foundational: both the plain-table links and the TUI open-in-browser keybind depend on the URL actually reaching the Setup. The data is already returned by Finnhub and the `url` crate is already a dependency (Cargo.toml:26); it just isn't plumbed.

Add a `url` field to the `NewsItem` deserialize struct (src/data/finnhub.rs:66-69) so Finnhub's per-article URL is no longer dropped. Change `get_top_catalyst` (finnhub.rs:199-218) to return the URL alongside the headline and catalyst, add a `catalyst_url: Option<String>` field to `Setup` (src/models/setup.rs:85-95), and set it where setups are assembled in the screener and in the `symbol` path (src/main.rs:107-110). This is shared groundwork for every link feature.
