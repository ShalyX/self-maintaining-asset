# PROJECT — NEAR SELF-MAINTAINING ASSET

## Original selected thesis
User-selected concept, 13 November 2026:
"Physical assets should be able to manage their own operating finances, build maintenance reserves, and coordinate their upkeep through programmable accounts and constrained AI agents."

First demonstration candidate: a revenue-generating solar-powered water point / commercial borehole.
Working folder name only: near-self-maintaining-asset. No public product name or brand is locked.

## Current phase
VALIDATION — active.
Engineering spike: VERIFIED IN LOCAL SANDBOX.
Product lock: NOT APPROVED. The complete user-facing product and end-to-end operator workflow are not yet verified.

## Completed work
- Fresh independent NEAR hackathon research and track review.
- Rust 1.95.0, cargo-near 0.23.0 and near-cli-rs 0.23.0 installed with Rust/Cargo homes on E:.
- Four disposable NEAR testnet accounts created for registry, owner, transferee and technician; registry balance verified at 10 NEAR.
- Official deadline, submission requirements and judging format recorded in HACKATHON.md.
- Winning Profile completed in WINNING_PROFILE.md.
- User selected this concept at HUMAN_IDEA_GATE.
- Current competition researched: MINTIQ, Nivry, ProofOps and water-sector products.
- NEAR Top Level Accounts implementation inspected at source level; its transfer/payout behavior is a technical constraint for this design.
- Project workspace created at E:\Hackathons\near-self-maintaining-asset.

## Active validation
See VALIDATION.md for the four mandatory gates and proof requirements.
Current working recommendation: MODIFY/NARROW the concept around physical-asset identity and asset-specific maintenance finance. The original thesis remains unchanged pending validation.

## Current build classification
ENGINEERING SPIKE VERIFIED IN LOCAL NEAR SANDBOX. The contract-level asset identity, child-account persistence, owner handover, financial controls and provisioning recovery are tested. No complete vertical slice, user-facing product or public testnet deployment has been verified.

## Product integrity checkpoint
LOCK REVISION: NONE — product not locked.
P0 TOTAL: UNKNOWN.
P0 VERIFIED: NOT YET MAPPED — the P0 inventory and total must be defined before product lock.
P0 MISSING/BLOCKED: NOT YET DEFINED.
DIFFERENTIATORS PRESERVED: provisional only; see VALIDATION.md.
END-TO-END USER JOURNEY: not built.
DRIFT VERDICT: PASS — the verified contracts retain the approved asset identity / operating account / maintenance reserve / ownership handover thesis; full product remains unverified.
NEXT ORIGINAL-PRODUCT TASK: build and validate the end-to-end maintenance workflow and prove the portable asset-history/reserve handover wedge with a target operator.

## Known blockers and dependencies
- No contract has been deployed yet; the testnet accounts and local credentials are ready.
- A fiat/mobile-money payment rail is not selected; do not imply live NGN customer payments.
- Custom per-asset account creation and safe NFT transfer passed the bounded local Sandbox spike; public testnet behavior and complete NEP-171 core-interface compliance remain unverified.
- Direct TLA account use is risky for a persistent treasury because transfer logic sweeps supported NEAR/FT balances to the outgoing payout account; see VALIDATION.md.

## Submission critical path
- Public repository: not created.
- Demo: not produced.
- Public X post: not created.
- Submission form: not completed.
- Deadline: 14 Nov 2026, 23:59 UTC.
- Recommended feature freeze: 14 Nov 2026, 12:00 UTC.

## Next action
Advance Gate 3 and Gate 4: define the minimum user-facing vertical slice for a real operator-supplied maintenance issue, diagnosis/work order, technician quote, human approval, completion evidence, reserve payout and persistent asset passport; validate the portable history/reserve handover need with a target operator. Keep testnet deployment, product lock and submission readiness separate until evidenced.

## Decision log
- 2026-11-13: User selected "The Self-Maintaining Asset" for validation.
- 2026-11-13: No product lock; differentiation and technical assumptions remain open.


## Validation checkpoint — 13 November 2026, 01:55 UTC

- Fresh kill-search confirmed close adjacent products: PaygOps, MajiPay, Avisers/Water Station and OmiChain. The NFT-plus-sub-account primitive is also explicitly present in NEAR official Track 10 starter example and is already implemented in NEAR Top Level Accounts. Generic asset NFT + agent treasury is not sufficiently differentiated.
- Validation thesis remains under test. Narrow differentiation candidate: portable operating history + asset-bound restricted maintenance reserve + safe authority handover across independent operators/service providers. Customer need for this continuity is unproven.
- Source review found both contract scaffolds, but the registry embeds contracts/asset-account/res/asset-account.wasm; the asset-registry/res directory is empty. The account contract has unit tests, but the registry integration path and transfer invariants are not yet verified.
- Rust 1.95 and wasm32 target are installed in WSL on E:, but the current PowerShell PATH does not expose cargo. WSL cargo/test attempts encountered prolonged uninterruptible I/O while accessing the E: mounted filesystem; attempts were stopped without claiming test success.
- No tests passed in this checkpoint. No WASM generated, no deployment, no sandbox transfer, no vertical slice.

## Next action
Resume the bounded technical spike using a Linux-local working copy/build target (avoid the slow WSL-mounted E: toolchain/cache path), compile and test asset-account, build its wasm, copy the verified wasm to asset-registry/res/asset-account.wasm, then compile/test registry. Add sandbox acceptance coverage for mint -> create child account -> deposit -> NFT transfer -> old-owner rejection -> new-owner payout -> balance/history continuity. Keep Gate 4 open until a real operator/customer interview or equivalent direct evidence validates the portability problem.


## Engineering continuation checkpoint — 13 November 2026

The user supplied the existing contract-system handoff and authorized callback/retry/economics hardening. No product redesign, OSIALI reuse, deployment, commit or push was authorized/performed. Existing source is implemented and prior handoff reports test passes, but the complete relevant current suite remains unverified.

Source audit found a critical NFT mint refund subsidy: standard internal_mint refunds the creator's child provisioning funds while registry pays the child. Draft corrects mint refund handling, separates per-asset retry credit, quotes only the owner-funded shortfall, uses checked empty initializer results and blocks uncertain provisioning. Adds bounded storage/gas safeguards and isolated fixture construction. Draft interface changes: get_asset_retry_cost(token_id), get_maintenance_payout_cost() with 0.01 NEAR attached for payout approval.

Infrastructure blocker: Ubuntu-24.04-Near root filesystem became read-only after disk I/O errors / temporarily unavailable E: drive. Cargo could not acquire its build lock, so no new assertions ran. Readable sources and known-good artifacts preserved before restarting only that distribution; startup now fails E_FAIL, and recovery distro startup timed out. No disk repair or broad WSL shutdown performed. Primary source has draft contract edits; added regression tests/isolated builder exist only in the C: recovered review package.

Review package: C:\Users\USER\Documents\Codex\2026-11-13\near-hackathon-engineering-continuation-handoff-continue-2\outputs\engineering-recovery
Read REVIEW.md and engineering-draft.diff there. Preserved production and child artifact hashes match initial inspection; they are not builds of the draft. Restore WSL/storage, reconcile draft with primary source, run serial unit/build/integration checks including protocol 85 and actual funding conservation, then review source/artifact changes before commit/push. Product lock, market evidence, complete P0 inventory, frontend/demo and submission gates remain open; this audit does not declare the full product verified.


## Engineering verification — 13 November 2026

The existing contracts were continued without redesign. Complete relevant local suite: registry 15/15 unit tests; asset-account 4/4; registry 3/3 serial isolated Sandbox integrations; documentation targets and strict Clippy/format checks passed. Production child/registry and isolated failure fixture built and hashes preserved. Source/artifact audit completed; verified contracts synchronized to the original repositories without altering the Git index, committing, pushing or deploying.

Review and proof package: C:\Users\USER\Documents\Codex\2026-11-13\near-hackathon-engineering-continuation-handoff-continue-2\outputs\validated-engineering

Corrections cover mint deposit-refund subsidy, asset-specific retry credit/exact top-up and legacy recovery, checked provisioning callbacks, bounded payout-storage charges, uncertain payout replay locks, and isolated fixture builds. API/deposit changes and remaining boundaries are documented in REVIEW.md. Unknown outcomes require reconciliation. Direct NFT transfer is supported; complete NEP-171 core interface compliance and reproducible release builds are not claimed.

PRODUCT INTEGRITY: approved NFT-backed asset account / revenue / maintenance / handover purpose retained. No new thesis or OSIALI architecture. This proves contract mechanisms and tested local journeys, not a complete product, real repair, customer demand, public deployment or submission readiness. Broader validation/product-lock gates remain open.

Next action: review the verified engineering package, then continue the existing product coverage/validation gates. No commit or push without review.


## Deadline triage — 14 November 2026

The live official NEAR event page was rechecked. Submissions close 14 November 2026 at 23:59 UTC (15 November 00:59 WAT). The original project was inspected on the connected PC.

**Recommendation: NO-GO for submitting the current project in its present form.** The source currently contains two separate local contract repositories with no commits/remotes, no root README, no frontend/operator interface, no public testnet deployment, no integrated technician maintenance workflow, no demo/X post and no submission. The 13 November engineering evidence reports 19 unit tests + 3 isolated Sandbox integrations passing, but these are local contract tests and were not rerun during this triage.

Product lock remains unapproved; Gate 3 remains open and Gate 4 remains MODIFY/NARROW. A contract-only entry would be a consequential reduction of the approved product and may not meet the judge-usable working-prototype requirement. Do not silently make that reduction.

Full evidence and the human decision gate are in DEADLINE_TRIAGE.md. No commit, push, deployment, public post, wallet disclosure or submission was performed. Phase remains VALIDATION; submission_ready=false; submission_status=NOT_STARTED.

Next action: await the user's explicit decision on whether to override scope for a clearly labeled contract-only experimental entry. Recommendation is to preserve scope and skip this deadline.


## User scope decision — 14 November 2026, 11:30 WAT

The user explicitly directed HQ to finish the real Self-Maintaining Asset and meet the NEAR submission deadline. The prior NO-GO recommendation and contract-only scope-reduction gate are superseded. Preserve the complete product thesis and build the integrated operator-to-technician maintenance journey, asset passport, reserve-funded payout and ownership handover. No P0 may be silently removed.

The official deadline is 14 November 2026, 23:59 UTC / 15 November 2026, 00:59 WAT. Approximately 13 hours 29 minutes remained at the user's 11:30 WAT instruction. Reserve time for testnet proof, judge-ready README, public repository, X demo with video/screenshots and the one-time official form. The one-time form payload must be shown to the user and receive final confirmation before submission.

Product contract created at PRODUCT.md. Phase remains VALIDATION until the remaining product gates and the integrated vertical slice have real proof. The current 19 unit tests + 3 Sandbox integrations are historical local evidence, not testnet proof. No public publication, X post, deployment or submission has been performed by this decision update.


## Verified continuation — 2026-11-14T19:34:40.230121+00:00

The complete maintenance product is implemented. Fresh validation: 6 account units + 20 registry units + 3 isolated Sandbox integrations passed. Creator-funded provisioning, retry recovery, authorization, history storage funding and full lifecycle were exercised. Dedicated original accounts remain untouched because their signing credentials could not be recovered. Read-only recovered VHD inspection found no credentials; the older E: image is unreadable. Separate 14 November testnet accounts were created and credentials backed up privately.

Public testnet full lifecycle passed, including human quote approval, completion evidence, handover, former-owner rejection and exact reserve payout. All 11 finalized transaction outcomes and final state are in `evidence/testnet-proof.json`. Deployed contract hashes match local artifacts; child has zero keys. App: https://self-maintaining-asset.vercel.app . GitHub repository created at https://github.com/ShalyX/self-maintaining-asset but implementation push remains pending.

Current gate: finish Meteor wallet acceptance and fresh asset registration, final visual QA and authentic demo capture. MyNearWallet is retired and has been replaced. No public post or submission yet. Human submission details supplied; team size remains pending. Final consent/payload confirmation is still required before submission.


## Release preservation — 14 November 2026, 20:41 WAT

A file-integrity scan found DEMO.md filled with null bytes. Windows reported E: NTFS errors 131 and repair event 130 at 20:17. The source, remaining text files and both deployed artifact hashes verified. The staged repository and current working files were copied to C: under the active Codex workspace work/self-maintaining-asset; the corrupt document was preserved separately and its demo plan reconstructed from verified product/evidence. Further release writes continue on C:, with E: left intact. No new product or replacement architecture was created.
