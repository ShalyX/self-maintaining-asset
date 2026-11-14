# SCORECARD

## Official rubric source and checked-at time
NEAR HACKATHON official page, checked 13 November 2026: https://nearhackathon.com/
The event says Claude Code and Codex screen submissions for reproducibility and README truth, followed by human judging. Current event-specific numeric weights were not found. Official weights: UNKNOWN.
Historical NEAR judging dimensions mentioned by the event: technical implementation, design, impact, originality, sustainability. These are guidance, not confirmed current weights.

## Original nine-factor ideation rubric — provisional post-kill-search reassessment
Scores are 1–5 per factor. No aggregate is calculated because no official weights are published and the purpose here is to expose weaknesses, not manufacture precision.

| Factor | Score | Evidence / current assessment |
|---|---:|---|
| Problem severity | 4 | Water-point downtime and maintenance-finance problems are documented; severity varies by operator type. |
| Evidence | 4 | CRS research and Nigerian community survey support the infrastructure problem; no direct customer interview yet. |
| Rubric fit | 5 | Strong fit with NFTs That Do Something; secondary Autonomous Business fit is possible but not locked. |
| Technology fit | 4 | NEP-171, per-asset accounts and contract-enforced reserves are integral; custom account transfer remains unproven. |
| Differentiation | 2 | MINTIQ overlaps generic NFT-agent treasury/jobs/payouts; MajiPay and Water Station overlap water payments/maintenance visibility. Physical-asset identity and lifecycle must carry the distinction. |
| Demo strength | 5 | Asset receives funds, protects reserve, gets a repair completed and paid, then changes operator while retaining its account/history. |
| Feasibility | 2 | Two-day window, missing Rust toolchain, and cross-contract account creation/transfer complexity. |
| Evidence potential | 4 | Contract invariants and testnet transactions can be shown; real NGN payment integration is not yet available. |
| Time-to-magic | 3 | A focused contract proof is feasible, but a full asset workflow is at risk if account creation becomes complex. |

## Evidence freshness and limitations
- MINTIQ website reachable; its testnet behavior and all claims are not independently verified.
- Nivry repository is a planning/research competitor, not a verified shipped product.
- Water-market products are adjacent competition; feature claims are from their own product pages.
- No direct operator interview; no evidence yet that small Nigerian operators want an on-chain account.
- Testnet NEAR deposits do not prove live NGN customer revenue.

## Qualitative standout review — not a score
- Understandable premise: potentially strong — "the asset has its own money and can pay for its own upkeep."
- Real differentiator: not yet proven; must be physical-asset identity + protected reserve + maintenance lifecycle + handover, not generic NFT-agent treasury.
- Demo payoff: clear if transfer preserves account balance/history and the repair is paid from the reserve.
- What a judge may doubt: why a Nigerian water operator needs blockchain rather than a bank account/CMMS; how the physical asset is linked to the NFT; whether the agent can steal funds; whether handover actually preserves value.
- What remains confusing: customer segment, fiat on-ramp, legal meaning of the NFT, and the exact account architecture.


## Engineering audit checkpoint — 13 November 2026

Technical readiness remains open: critical creation subsidy found, fixes drafted, integration validation blocked by WSL/storage failure. No score increase, full-product verification or deployment readiness is justified. Continue with preserved source/review package and serial economic/ownership/recovery proof after infrastructure recovery.


## Engineering verification — 13 November 2026

Contract mechanism proof improved: 19 unique unit tests and 3 integrated local Sandbox scenarios pass, with strict lint/format gates and artifact hashes. Evidence: C:\Users\USER\Documents\Codex\2026-11-13\near-hackathon-engineering-continuation-handoff-continue-2\outputs\validated-engineering/REVIEW.md. No invented numeric score increase; product completeness, differentiation/customer evidence, deployment and submission gates remain open.
