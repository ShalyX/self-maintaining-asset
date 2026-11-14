# Release hygiene

Canonical product: The Self-Maintaining Asset. Canonical app: https://self-maintaining-asset.vercel.app . Track: NFTs That Do Something.

Credentials, wallet recovery phrases, local Vercel tokens, browser storage, backups and failure-build scratch are excluded from Git. Public RPC evidence contains account IDs, public keys, transaction data and contract views only. The repository intentionally includes the tested WASM artifacts and test logs.

The demo evidence is a labelled testnet fixture. The frontend reads real final NEAR state and signs through an external wallet. No IoT sensing, LLM integration, autonomous spending, physical repair or mainnet readiness is claimed.

The original nested Git metadata is retained in a local ignored backup. Root Git now represents the complete product. Contract scaffold workflows under each contract are inactive nested files; root instructions are authoritative. The scaffold Docker reproducible-build metadata has not been validated.

Fresh desktop/mobile screenshots, wallet transaction acceptance, final public links and submission status are recorded as each check completes in EVIDENCE.md. A live deployment is separate from hackathon submission.
