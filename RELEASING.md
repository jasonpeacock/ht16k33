# Releasing new crates

1. Run `cargo update` to refresh dependencies.
1. Run `cargo outdated` and fix any advice.
1. Run `touch src/lib.rs && cargo clippy` and fix any advice.
1. Run `cargo clean && cargo build && cargo test` to double-check everything is happy.
1. Update the version info in `Cargo.toml` as appropriate.
1. Dry-run the publish: `cargo publish --dry-run --allow-dirty`
1. Git push the change, wait for CI to pass.
1. Tag the commit & push it: `git tag vX.Y.Z; git push --tags`
1. Publish the crate: `cargo publish`
