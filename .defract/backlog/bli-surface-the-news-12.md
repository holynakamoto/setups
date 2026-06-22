---
id: bli-surface-the-news-12
rawText: ''
title: Add styled link affordance for news in the detail panel
type: task
epic: Hyperlink interaction
module: ui
size: s
labels:
- news
dependsOn:
- bli-thread-finnhub-news-10
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:01:30Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:00:36Z
- type: grooming_completed
  timestamp: 2026-06-22T15:01:30Z
  summary: Cleaned title, assigned Hyperlink interaction epic, ui module, size s, news label, and wired depends_on to bli-thread-finnhub-news-10.
---

Captured by chat agent during this chat: The redesign must accommodate the hyperlink, which means giving the link a deliberate place and visual treatment in the detail view rather than plain text.

Update the detail panel (dashboard.rs:179-262) so the selected setup's news shows a clear, styled link affordance — e.g. the headline rendered as a link-styled span with a visible hint that it is openable. Depends on catalyst_url from the plumbing epic.
