---
id: bli-create-homebrew-tap-16
rawText: ''
title: Create Homebrew tap with setups formula
type: task
epic: Homebrew distribution
module: ci
size: m
labels:
- release
dependsOn:
- bli-add-a-github-15
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:02:58Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:02:16Z
- type: grooming_completed
  timestamp: 2026-06-22T15:02:58Z
  summary: Cleaned title, assigned Homebrew distribution epic and ci module from corpus, sized m, labeled release, wired depends_on to bli-add-a-github-15
---

Captured by chat agent during this chat: The consumption half of brew packaging; turns the published artifacts into a one-line install. Requires the public GitHub repo + tap the builder selected.

Stand up a public homebrew-tap repository with a `setups` formula that downloads the per-arch release tarball by url + sha256 and installs the binary, so users can `brew install <owner>/tap/setups`. Document the install in the README. Depends on the release workflow producing artifacts.
