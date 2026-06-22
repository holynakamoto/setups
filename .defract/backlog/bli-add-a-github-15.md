---
id: bli-add-a-github-15
rawText: ''
title: Publish macOS release binaries to GitHub Releases via CI
type: task
epic: Homebrew distribution
module: ci
size: m
labels:
- release
- ci
groomingStatus: completed
createdAt: 2026-06-22T15:00:36Z
groomedAt: 2026-06-22T15:02:12Z
events:
- type: grooming_started
  timestamp: 2026-06-22T15:01:34Z
- type: grooming_completed
  timestamp: 2026-06-22T15:02:12Z
  summary: Cleaned title, typed as task, placed in existing Homebrew distribution epic under ci module, sized m, labeled release+ci
---

Captured by chat agent during this chat: Builder chose prebuilt-binary distribution; this is the publishing half. It is independent infra with no source-code dependency on the other epics, so it can proceed in parallel.

Create a CI workflow that, on a version tag push, cross-compiles release binaries for macOS (aarch64-apple-darwin and x86_64-apple-darwin), packages them as tarballs, computes sha256 sums, and attaches them to a GitHub Release. Establishes the versioned, hosted artifacts a formula consumes.
