# asset-account

Dedicated keyless account: deterministic revenue allocation and registry-authorized maintenance payouts.

The complete product, build instructions, verified transactions and limitations are in the [root README](../../README.md) and [judge walkthrough](../../JUDGE_WALKTHROUGH.md).

Run `python3 scripts/validate.py` from the repository root to rebuild and test both contracts, embed the exact child, isolate the invalid-WASM fixture, and promote artifacts only after all checks pass. Rust 1.97.1, NEAR SDK 5.30.1; Sandbox protocol 85.

Verified 14 November 2026: 6 asset-account unit tests, 20 registry unit tests and all 3 isolated Sandbox scenarios passed. Public testnet evidence is in `evidence/testnet-proof.json`. No mainnet readiness or physical repair is claimed.
