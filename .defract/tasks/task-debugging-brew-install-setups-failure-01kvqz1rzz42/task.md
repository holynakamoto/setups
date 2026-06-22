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

# Enable brew install via release CI and Homebrew tap

## What We're Building

A complete Homebrew distribution pipeline so users can install setups with a single `brew install` command. This involves two pieces that must land in order: a tag-triggered CI workflow that compiles and publishes macOS binaries to GitHub Releases, followed by a public Homebrew tap repository whose formula downloads those published binaries by URL and checksum.

## Expected Outcome

- Running `brew tap holynakamoto/setups && brew install setups` on a Mac installs the tool without errors
- Pushing a version tag (e.g., `v0.1.0`) automatically triggers a build that produces arm64 and x86_64 macOS binaries attached to a GitHub Release
- The Homebrew formula contains real checksums that match the published release tarballs, replacing the current placeholders
- Users on Apple Silicon and Intel Macs install from the same formula with no manual steps

## Phase Outcomes

- **Phase 1: Automated release pipeline** — Every tagged release automatically builds macOS binaries for both chip architectures, updates the Homebrew formula with correct checksums, and publishes a GitHub Release — making the tool installable via `brew install` without any manual steps.

## Out of Scope

- Linux or Windows binary distribution — macOS only for this pipeline
- Automated version bumping, changelog generation, or release note templating
- Changes to the application source code — this is packaging and CI infrastructure only
- Creating a separate `homebrew-setups` tap repository — the formula remains in the main repo; the short-form `brew tap holynakamoto/setups` install command requires that follow-up (see Technical Notes)

## Scope Summary

**Size:** 11 requirements, 6 acceptance criteria, 1 implementation phase
**Key decisions:**
- Use native macOS runners per architecture (ARM runner for aarch64, Intel runner for x86_64) rather than cross-compilation, avoiding toolchain complexity
- Automate formula checksum updates inside the release workflow via `sed`, eliminating the manual post-release step
- Formula stays in the main `setups` repo; full-URL tap form required until a `homebrew-setups` repo is created
**Biggest risk:** The formula-update commit pushed back to `master` by the workflow can race with concurrent development commits; low probability given release cadence, mitigated by a `git pull` before commit.

## Context

`Formula/setups.rb` already exists in this repo with the correct URL pattern, `install`, `caveats`, and `test` blocks, but uses `PLACEHOLDER_SHA256_AARCH64` and `PLACEHOLDER_SHA256_X86_64` strings that cause `brew install` to fail checksum verification. No `.github/` directory exists — there is no release CI today, so the URLs in the formula are also currently 404 (no binaries are published). The fix requires two coordinated pieces: a GitHub Actions workflow that builds platform-specific binaries on every `v*` tag and publishes them to GitHub Releases, and an automated step in that same workflow that replaces the placeholder strings in `Formula/setups.rb` with real SHA256 values and commits the update back to `master`.

## Requirements

### Release CI Workflow

- R1: A GitHub Actions workflow file at `.github/workflows/release.yml` triggers on `push` events where the tag ref matches `v*.*.*`.
- R2: The workflow defines a build matrix with two jobs: one running on `macos-14` targeting `aarch64-apple-darwin`, and one on `macos-13` targeting `x86_64-apple-darwin`.
- R3: Each build job installs the Rust target with `rustup target add <target>` then builds with `cargo build --release --target <target>`.
- R4: Each build job creates a `.tar.gz` archive containing only the `setups` binary, named `setups-<target>.tar.gz` (e.g., `setups-aarch64-apple-darwin.tar.gz`).
- R5: Each build job computes the SHA256 of its tarball using `shasum -a 256` and uploads both the tarball and a `.sha256` text file as GitHub Actions artifacts.
- R6: A `publish` job runs after both build jobs succeed (`needs: [build]`) and downloads all artifacts.
- R7: The `publish` job reads both `.sha256` files, then uses macOS `sed -i ''` to replace `PLACEHOLDER_SHA256_AARCH64` and `PLACEHOLDER_SHA256_X86_64` in `Formula/setups.rb` with the real hex strings.
- R8: The `publish` job configures a git identity, pulls the latest `master`, commits the updated formula to `master` with message `chore: bump formula checksums for <tag>`, and pushes using `GITHUB_TOKEN`.
- R9: The `publish` job creates a GitHub Release for the pushed tag via `gh release create` and uploads both `.tar.gz` files as release assets.

### Homebrew Formula

- R10: `Formula/setups.rb` retains its current structure (arch-conditional `on_macos`/`on_arm`/`on_intel` blocks, `install`, `caveats`, and `test` blocks); only the two placeholder inline comments are removed as the values are now automated.
- R11: The formula's `test` block passes `--help` to the binary and matches `"setups"` in output, verifying the binary is installed and executable.

## Acceptance Criteria

- [ ] Pushing a `v*.*.*` tag triggers the workflow; the Actions tab shows two parallel build jobs and a `publish` job that runs after both complete.
- [ ] Both `setups-aarch64-apple-darwin.tar.gz` and `setups-x86_64-apple-darwin.tar.gz` appear as assets on the GitHub Release created for the tag.
- [ ] After the workflow completes, `Formula/setups.rb` on `master` contains no occurrences of `PLACEHOLDER_SHA256_AARCH64` or `PLACEHOLDER_SHA256_X86_64`.
- [ ] The SHA256 values committed to `Formula/setups.rb` match the output of `shasum -a 256` run locally against the downloaded release tarballs.
- [ ] `brew install` from the tap completes without checksum errors on an Apple Silicon Mac.
- [ ] `brew test holynakamoto/setups/setups` passes (the `--help` invocation exits 0 and matches `"setups"`).

## Implementation Phases

### Phase 1: Automated release pipeline

**Scope:** Create the GitHub Actions release workflow and remove the placeholder comments from the formula so every tagged release automatically produces installable macOS binaries with correct checksums — no manual steps required after pushing a tag.

**Files:**
- `.github/workflows/release.yml` — new; tag-triggered pipeline with parallel arch builds and a `publish` job that updates the formula and creates the GitHub Release
- `Formula/setups.rb` — remove the two placeholder comment lines (lines 9 and 15); the CI workflow writes real SHA256 values at release time

**Verification:**
- [ ] `python3 -c "import sys, yaml; yaml.safe_load(sys.stdin)" < .github/workflows/release.yml` exits 0 (valid YAML)
- [ ] `grep -r "PLACEHOLDER" Formula/setups.rb` returns no matches
- [ ] After pushing a test tag, all three jobs (two build + publish) complete green in the Actions tab
- [ ] Both `.tar.gz` assets appear on the GitHub Release page for the test tag
- [ ] `Formula/setups.rb` on `master` contains real 64-character hex SHA256 strings after the workflow run

**Estimated effort:** Medium

## Edge Cases

- **`rustup target add` on native runner**: Adding `x86_64-apple-darwin` on an Intel `macos-13` runner is a no-op that exits 0 — no special handling needed, but keeping the `rustup target add` call makes the job symmetric and future-proof.
- **Formula commit races a concurrent push to `master`**: The `publish` job runs `git pull --rebase` before committing to pick up any concurrent changes; if the rebase fails, the workflow errors rather than silently corrupting the formula.
- **Two release tags pushed in rapid succession**: Each workflow run is scoped to its triggering tag via `${{ github.ref_name }}`; the second `publish` job's `git pull` will see the first job's commit and rebase cleanly in the typical case.
- **`FINNHUB_API_KEY` absent at install time**: The binary exits with an error on first invocation; the formula's `caveats` block already explains the requirement, so no CI handling is needed.
- **`brew test` without a live API key**: The test only invokes `--help`, which exits before any API call, so it always passes regardless of environment configuration.

## Technical Notes

**Runner strategy**: The workflow uses `macos-14` (Apple Silicon) for the `aarch64-apple-darwin` target and `macos-13` (Intel) for `x86_64-apple-darwin`. Native builds on matching hardware avoid cross-compilation linker configuration and the `cross` crate.

**Workflow permissions**: The workflow requires `permissions: contents: write` so `GITHUB_TOKEN` can push the formula-update commit and create the GitHub Release. Without this, both operations fail with HTTP 403. This must be set at the job or workflow level in the YAML.

**Repository setting prerequisite**: GitHub repository Settings → Actions → General → Workflow permissions must be set to "Read and write permissions" for `GITHUB_TOKEN` to have push access. Without this repository-level setting, even a correctly-scoped `permissions:` block in the YAML will be rejected.

**macOS `sed` syntax**: The formula update uses `sed -i '' 's/PLACEHOLDER_SHA256_AARCH64/<sha>/g' Formula/setups.rb` (BSD `sed` on macOS — the empty string after `-i` is required). GNU `sed` syntax (`sed -i 's/...'`) would fail on macOS runners.

**`reqwest` and TLS**: The crate uses `rustls-tls` (not system OpenSSL), so no `brew install openssl` or `pkg-config` setup is needed in CI before building.

**Tap install command**: `brew tap holynakamoto/setups` resolves to `github.com/holynakamoto/homebrew-setups` by Homebrew convention — a repo that does not exist. Until a separate `homebrew-setups` tap repo is created, the correct sequence is:
```
brew tap holynakamoto/setups https://github.com/holynakamoto/setups
brew install holynakamoto/setups/setups
```
The Expected Outcome in the approved intent check uses the short form; that form becomes valid when a `homebrew-setups` repo is created as a follow-up task.

### Dependencies

- GitHub repository Settings → Actions → General → Workflow permissions must allow read and write access before the release workflow can push the formula commit or create a release.
