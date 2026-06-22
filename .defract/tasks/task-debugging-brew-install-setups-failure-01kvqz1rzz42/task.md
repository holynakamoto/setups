---
defract:
  id: task-debugging-brew-install-setups-failure-01kvqz1rzz42
  type: improvement
  status: active
  stage: scope
  phase: 0
  total_phases: 1
  priority: normal
  source: manual
  branch_strategy: worktree
  mode: human-in-the-loop
  created_by: holynakamoto
  assignee: holynakamoto
---

## Story Brief

# Enable brew install via release CI and Homebrew tap

## What We're Building

A complete Homebrew distribution pipeline so users can install setups with a single `brew install` command. This involves two pieces that must land in order: a tag-triggered CI workflow that compiles and publishes macOS binaries to GitHub Releases, followed by a public Homebrew tap repository whose formula downloads those published binaries by URL and checksum.

## Expected Outcome

- Running `brew tap holynakamoto/setups && brew install setups` on a Mac installs the tool without errors
- Pushing a version tag (e.g., `v0.1.0`) automatically triggers a build that produces arm64 and x86_64 macOS binaries attached to a GitHub Release
- The Homebrew formula contains real checksums that match the published release tarballs, replacing the current placeholders
- Users on Apple Silicon and Intel Macs install from the same formula with no manual steps

## Out of Scope

- Linux or Windows binary distribution — macOS only for this pipeline
- Automated version bumping, changelog generation, or release note templating
- Changes to the application source code — this is packaging and CI infrastructure only
