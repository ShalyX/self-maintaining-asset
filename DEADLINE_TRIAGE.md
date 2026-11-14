> Historical snapshot. Superseded by the full-product execution decision and verified continuation in PROJECT.md. Current evidence: JUDGE_WALKTHROUGH.md and evidence/.

# NEAR HACKATHON — DEADLINE TRIAGE

**Snapshot:** 14 November 2026
**Official close:** 14 November 2026, 23:59 UTC = 15 November 2026, 00:59 WAT
**Project:** The Self-Maintaining Asset
**Phase:** VALIDATION
**Product lock:** NOT APPROVED
**Recommendation:** NO-GO for submission in its current form. A contract-only scope override requires explicit user approval.

## 1. Official submission requirements

Official source: https://nearhackathon.com/ (checked 14 November 2026).

Required:
1. Build a working NEAR-focused project.
2. Publish the repository; judges cannot access a private repository.
3. Publish a public X demo post saying the project was built at NEAR HACKATHON, tagging @NEARProtocol and including #NEARHackathon, with a demo video or screenshots.
4. Submit both the public repository URL and public X post URL through the official form before the deadline.
5. Provide a NEAR account address for prize payout.

The official page says Claude Code and Codex screen projects for whether they run and do what the README promises before human judging. The README must therefore give reproducible, truthful steps.

## 2. Reconciled implementation status

### Verified engineering (historical, not rerun during this triage)

The 13 November engineering review reports:
- Registry: 15/15 unit tests passed.
- Asset account: 4/4 unit tests passed.
- Registry: 3/3 serial isolated Sandbox integrations passed.
- Strict Clippy with warnings denied and formatting checks passed.
- Production child, registry and isolated failure fixture were built; SHA256 evidence is recorded in the review package.

Review:
C:\Users\USER\Documents\Codex\2026-11-13\near-hackathon-engineering-continuation-handoff-continue-2\outputs\validated-engineering\REVIEW.md

Validation results:
C:\Users\USER\Documents\Codex\2026-11-13\near-hackathon-engineering-continuation-handoff-continue-2\outputs\validated-engineering\validation-results.json

This proves the tested contract mechanisms in local NEAR Sandbox. It does not prove a public testnet deployment, full product, live operator workflow or customer adoption. These tests were not rerun as part of this deadline triage.

### Actual source and repository inspection (14 November)

- Original workspace: E:\Hackathons\near-self-maintaining-asset.
- No root-level Git repository.
- The asset-registry and asset-account folders are separate nested Git repositories.
- Both nested repositories report **no commits yet**, have no configured remotes, and contain staged/unstaged source changes.
- No matching project repository was found in the connected GitHub account's accessible repository list.
- No root-level README or consolidated judge-facing build/run guide.
- Workspace contains Rust contract source and artifacts only; no frontend, web application, dashboard, or end-to-end operator interface was found.
- No public testnet deployment is recorded or verified.
- No public demo/video or X post exists in the project checkpoint.
- No submission has been made.

### Product and validation gates

| Gate | Actual status | Consequence |
|---|---|---|
| Asset NFT + dedicated account + transfer authority | VERIFIED IN LOCAL SANDBOX | Strong technical proof, not a deployed product |
| Contract-level reserve/payout safety | VERIFIED IN LOCAL SANDBOX | Agent and product integration still absent |
| Operator-to-technician maintenance workflow | OPEN | No integrated issue → diagnosis → quote → approval → completion evidence → payout → passport journey |
| Market differentiation | MODIFY/NARROW | Portable history/reserve/authority handover remains unvalidated by a direct operator |
| Product lock | NOT APPROVED | Target user, P0 scope and final workflow remain undefined |
| Public testnet deployment | NOT DONE / NOT VERIFIED | No live chain journey for judges |
| Judge-facing product/demo | NOT DONE | No visible product payoff to record |
| Public repository | NOT READY | No commits/remotes and no consolidated repository |
| X demo post + official form | NOT DONE | Mandatory submission artifacts absent |

Technical limitation: the current NFT contract supports direct transfer and token/metadata views, but does not implement nft_transfer_call. Do not claim complete NEP-171 core-interface compliance.

## 3. Submission viability

**NO-GO for the current project in its current form.**

This is not a judgment that the idea is bad. It is a readiness decision based on mandatory evidence:
- The current deliverable is two local contract repositories, not a judge-usable working product.
- No testnet deployment exists.
- No complete operator workflow exists.
- No public repository or reproducible root README exists.
- No demo recording or required public X post exists.
- Product lock and validation are still open.

Given the open validation gates and missing judge-facing product, the remaining build window is not a credible basis for responsibly completing, integrating, deploying, testing, documenting, recording, publishing and submitting the full approved product. Rushing to label the contract subsystem as the complete Self-Maintaining Asset would misrepresent the current evidence and violate the Operator's product-integrity rules.

## 4. Consequential scope override — human decision required

The only plausible way to attempt an entry before this deadline is to materially reduce the entry to a **contract-only experimental prototype**. That would remove or defer the approved user-facing maintenance workflow, technician journey, asset passport experience and market validation.

This would not be the complete Self-Maintaining Asset. It may also fail the event's working-prototype/reproducibility screen if the judge cannot deploy and interact with it from the public README.

**Recommendation: preserve the approved product scope and do not submit this project for this deadline.**

If the user explicitly authorizes the contract-only override, the permitted next step would be to prepare a clearly labeled experimental contract prototype and assess whether a truthful, reproducible submission can still be assembled. No public repository creation/push, deployment, X post, wallet disclosure or form submission is authorized by this triage.

## 5. If preserving scope: next validation/build sequence

1. Define and lock the target operator, user workflow and P0 inventory only after the existing validation gates pass.
2. Build the actual operator-to-technician vertical slice: operator observation → constrained diagnosis/work order → technician quote → human approval → completion evidence → testnet payout → persistent asset history/reserve.
3. Deploy the registry and child contracts to NEAR testnet and capture transaction IDs, balances and owner-handover evidence.
4. Complete the required NFT interface or clearly document the supported subset.
5. Build the judge-facing interface and a reproducible reset/demo path.
6. Consolidate into one public repository with a root README, exact setup/run commands, expected outcomes and test evidence.
7. Run product-integrity, proof, QA, hygiene, red-team and judge simulation.
8. Produce the actual demo video, publish the required X post and submit only after the full audit and human authorization.

## 6. Safety and integrity boundaries

- Do not change the approved thesis or silently reduce scope.
- Do not claim public testnet/mainnet deployment, live NGN revenue, real sensors, real technicians, customer adoption or complete NEP-171 compliance.
- Do not commit, push, deploy, publish, post or submit as part of this triage.
- Keep the phase at VALIDATION, product_locked=false, submission_ready=false and submission_status=NOT_STARTED until new evidence and any required human decision exist.

## 7. Sources

- Official event requirements and deadline: https://nearhackathon.com/
- Official starter prompts and submission guidance: https://github.com/nearai/near-hackathon-prompt-examples
- Existing engineering review and logs: validated-engineering package listed above.
- Existing project state and gate details: .hackathon/state.json, PROJECT.md, VALIDATION.md, EVIDENCE.md, contract source and READMEs.
