# Final engineering review — 14 November 2026

## Verified release

6 account unit tests, 20 registry unit tests, 3 isolated Sandbox integrations, frontend production build and lint passed. The clean Ubuntu GitHub CI run independently rebuilt and validated the complete suite: [run 43633555968](https://github.com/ShalyX/self-maintaining-asset/actions/runs/43633555968). Public testnet outcomes, code hashes and final state are in `evidence/testnet-proof.json`; browser acceptance is separate in `evidence/browser-qa.json`.

## Callback and recovery correctness

Provisioning requires one result and accepts only empty successful initializer output through `promise_result_checked(0, 0)`. A definite failed receipt is `FAILED`; oversized/unexpected output is `PROVISIONING_UNKNOWN` and cannot be treated as success or reusable funding. No deprecated promise-result API remains. Invalid result counts abort and keep provisioning locked.

Payouts lock the work order before cross-contract execution. Only the child's explicit `true` marks it paid; explicit `false` means reserve restoration; invalid/oversized/failed callbacks preserve the lock as unknown. Unknown outcomes require investigation and must not be replayed. A native transfer returns empty bytes; the child's private callback restores reserve only when that transfer reports an error.

## Funding and storage

Creator funding covers current embedded child-code storage, a 0.25 NEAR operating buffer and a 0.1 NEAR NFT/registry storage allowance. Underfunded creation fails before minting. The mint helper does not refund child provisioning funds. Successful provisioning consumes its per-asset credit; definite failed atomic creation refunds to the registry asynchronously. Retry quotes the exact current shortfall against that asset's credit (minimum 1 yoctoNEAR), forwards all attached funding and preserves surplus in the child. Legacy assets without recorded credit require full owner funding. Repeated failures do not create claims on shared registry funds.

Ownership history previously grew with only a one-yocto transfer deposit. The release now prepays history from the creator's remaining asset-specific allowance, bounds memo bytes, and charges actual storage growth on each transfer. The current owner can refill that asset's history credit. Transfer has atomic rollback if its credit is insufficient; its dedicated account and revenue balances remain unchanged.

Work-order creation pays 0.05 NEAR; payout bookkeeping pays 0.01 NEAR. Input byte limits bound record growth. These are retained prototype storage allowances, not a refundable storage-management standard or a production pricing model. Creation measures and bounds actual NFT storage cost. The serial integration budget accounts for the work-order fee and checks that permissionless creation does not draw from shared treasury funding.

## Access, arithmetic and gas

Current NFT owner authorization is checked at each assignment, quote approval, payout and transfer. Only the assigned technician can quote and submit completion. Payout recipient and amount must match the approved quote, and reserve sufficiency is checked in the child. Pending and paid records prevent payout replay across ownership transfer. The child initializes only under its parent registry and accepts payouts only from that registry; its deployed key list is empty.

Funding, total deposits and allocation counters use checked arithmetic; percentage bounds prevent invalid splits. Release overflow checks are enabled. Provisioning and payout require at least 100 Tgas. Actual child creation, callback completion, failed provisioning, code-recovery retry, technician payout and handover passed on Sandbox; the complete successful lifecycle also passed testnet.

## Known limits

This is an independent testnet prototype. The registry account retains an upgrade key; recovery therefore trusts its operator. Child accounts are keyless and have no general upgrade path. Unknown promise outcomes remain locked rather than guessing whether funds moved. Operating funds are accounted for and preserved; general operating withdrawals are not implemented. No full NEP-171 transfer-call/resolver support, decentralized evidence hosting, live sensors, LLM integration, fiat rail, physical repair verification or mainnet readiness is claimed.

The browser's uploaded evidence is session-local; its hash/reference is durable on chain. The demo repair file is explicitly simulated. No operator/customer adoption is claimed. Public submission and X publication require separate confirmation and verification.

## Independent build identity

The final clean Ubuntu run [43635827710](https://github.com/ShalyX/self-maintaining-asset/actions/runs/43635827710) passed all checks and exported its logs and hashes to `evidence/ci-final/`. Its WASM byte hashes differ from the validated WSL release embedded/deployed on testnet. Both build outputs passed the full Sandbox suite. Cross-environment byte reproducibility is unverified; no cause or reproducible-build attestation is asserted. The repository release artifacts intentionally remain the exact binaries verified against deployed code hashes in `evidence/testnet-proof.json`.
