# Build artifacts

Built on 14 November 2026 with Rust 1.97.1, locked Cargo dependency graphs, target wasm32-unknown-unknown, release profile.

- asset_account.wasm: dedicated asset-account contract.
- asset_registry.wasm: registry/NFT/work-order contract; embeds the asset-account WASM from contracts/asset-registry/res/asset-account.wasm.

Rebuild instructions are in the repository-root README. SHA-256 digests are recorded in SHA256SUMS.txt.
