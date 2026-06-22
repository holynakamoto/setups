# Proven Workflows

## Workflows

- [01KVRB3DW02JGQX7S6K33RBPZ7] **- **Rust macOS release workflow pattern: parallel native builds → publish j...** -- - **Rust macOS release workflow pattern: parallel native builds → publish job** — Tag-triggered (`v*.*.*`) workflow with: (1) `build` matrix job running on `macos-14` (aarch64) and `macos-13` (x86_64) in parallel — each runs `rustup target add`, `cargo build --release --target`, creates `.tar.gz`, computes SHA256 via `shasum -a 256 | awk '{print $1}'`, uploads both as artifacts; (2) `publish` job (`needs: [build]`) downloads all artifacts, reads `.sha256` files into env vars, applies BSD `sed -i ''` to update the formula, commits to master, then runs `gh release create` with both tarballs. Workflow needs `permissions: contents: write`. [source: task-debugging-brew-install-setups-failure-01kvqz1rzz42, importance: 0.6]. [source: task-debugging-brew-install-setups-failure-01kvqz1rzz42, importance: 0.6]

