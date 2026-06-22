# Project Facts

## Tech Stack


## Conventions

- [01KVRB398RAMT60A1KM1HV4C5F] **- **Homebrew formula PLACEHOLDER sha256 values must be retained as CI sed sub...** -- - **Homebrew formula PLACEHOLDER sha256 values must be retained as CI sed substitution targets** — When CI automates checksum updates via `sed`, the formula must contain the exact placeholder strings (e.g., `PLACEHOLDER_SHA256_AARCH64`) that sed will replace. Removing them pre-CI makes the sed commands no-ops and leaves the formula without real sha256 values. Only remove placeholder *comment* lines (e.g., `# Replace PLACEHOLDER_SHA256...`) — not the placeholder values themselves. [source: task-debugging-brew-install-setups-failure-01kvqz1rzz42, importance: 0.6]. [source: task-debugging-brew-install-setups-failure-01kvqz1rzz42, importance: 0.6]
- [01KVRB35N5CG9PSAZTCGDWTGPP] **- **BSD `sed -i ''` (not GNU sed) required on macOS GitHub Actions runners** ...** -- - **BSD `sed -i ''` (not GNU sed) required on macOS GitHub Actions runners** — macOS runners (macos-14, macos-13) ship with BSD sed. In-place substitution requires an explicit empty string backup suffix: `sed -i '' 's/FIND/REPLACE/g' file`. GNU sed syntax `sed -i 's/...' file` (no empty string) fails on macOS with "invalid command code". [source: task-debugging-brew-install-setups-failure-01kvqz1rzz42, importance: 0.6]. [source: task-debugging-brew-install-setups-failure-01kvqz1rzz42, importance: 0.6]

## Patterns


