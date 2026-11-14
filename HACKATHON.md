# NEAR HACKATHON — VERIFIED BRIEF

## Event
NEAR HACKATHON, 1–14 November 2026.

## Deadline and timezone
Development and submission close: **14 November 2026, 23:59 UTC** (15 November 2026, 00:59 WAT).
Results announced by 18 November 2026, 23:59 UTC.
Checked: 13 November 2026.

## Eligibility and team constraints
- Solo builders and teams may participate.
- No pre-registration is required to start building.
- One submission per project.
- One person may submit up to two projects only if they are independent; each README must name the other project.
- Submission requires a NEAR account address for prize payout; no wallet connection is needed to submit.

## Selected track — locked

**Primary: NFTs That Do Something (Track 10).** The product uses an NEP-171 operational passport, a dedicated asset sub-account, a protected reserve and portable service history through NFT handover.

**Secondary: None.** Do not claim Autonomous Business (Track 4): this prototype does not implement an autonomous business agent, NEAR Intents, Chain Signatures or an external payment rail.

## Judging
- Claude Code and Codex screen every submission first for whether it runs and does what its README promises.
- Human judges make final decisions.
- Current event-specific numeric weights were not found; record official weights as UNKNOWN.
- The event page says previous NEAR hackathons evaluated technical implementation, design, impact, originality, and sustainability. This is historical guidance, not a confirmed current weighted rubric.
- Current jury listed by the event: Illia Polosukhin, Eddy Lazzarin, Mert Mumtaz, Avichal Garg, Kain Warwick, Haseeb Qureshi, and Stepan Gershuni.

## Submission critical path
- [ ] Public GitHub repository.
- [ ] Working prototype with reproducible README instructions.
- [ ] Public X demo post explicitly saying it was built at NEAR HACKATHON.
- [ ] Tag @NEARProtocol and include #NEARHackathon.
- [ ] Attach demo video or screenshots to the X post.
- [ ] Submit repository URL and X post URL through the event form before deadline.
- [ ] Record NEAR payout account in the form.
- [ ] Verify submission confirmation/receipt.
- Recommended feature freeze: 14 November 2026, 12:00 UTC (13:00 WAT), leaving 12 hours for final QA and submission.

## Technology requirements
No single mandatory contract framework was identified. Track 10 explicitly points to NEP-171 and sub-accounts. Testnet is suitable for a working prototype. Any mock or simulated external input must be clearly labelled.

## Official resources
- Event: https://nearhackathon.com/
- Official brief: https://nearhackathon.com/HACKATHON.md
- Track prompts: https://github.com/nearai/near-hackathon-prompt-examples
- NEAR smart-contract agent skill: https://github.com/near/agent-skills/tree/main/skills/near-smart-contracts

## Disqualification / submission risks
- Private GitHub repository.
- Missing public X post or missing required tag/hashtag.
- Missing demo video/screenshots.
- Missing repository or X post link in submission form.
- README claims behavior the judge cannot reproduce.
- Late submission.
- Any claim of mainnet or production operation when only testnet is implemented.

## Unknowns
- Exact form fields and whether any additional evidence uploads are requested: inspect form before final day.
- Current event-specific rubric weights: not published in the sources checked.


## Final submission API — verified 14 November 2026

Official endpoint: POST https://nearhackathon.com/api/applications. Form: https://nearhackathon.com/#apply. The submission is one-time per repository and cannot be edited directly; corrections go through https://nearhackathon.com/request-changes.

Required JSON fields (no extras):
- requestId: one lowercase UUID generated once and reused on retries.
- projectName: 2–80 characters.
- leadName: 2–80 characters.
- email: valid email, max 160 characters.
- size: exact enum; solo builder uses "1".
- tracks: array of official track IDs. This project uses ["nfts"].
- nearAddress: a MAINNET named NEAR account or 64-character implicit account, lowercase. Never use a testnet demo account for prize payout and never invent an address.
- projectUrl: public repository URL, max 300 characters; each repository may be submitted once.
- demoUrl: public X demo-post URL, max 300 characters.
- idea: 20–2000 characters describing the problem, product and how NEAR fits.
- consent: true only after the human agrees to the exact notice: "I agree that the organizers may use these details to review our submission and contact us about this hackathon."

Do not include a website field; the official API treats a non-empty website field as a spam trap.

Before POST:
1. Make the repository public only after explicit human approval.
2. The human publishes the X demo post themselves; the post must say it was built at NEAR HACKATHON, tag @NEARProtocol and #NEARHackathon, and attach the video or screenshots.
3. Collect the human's lead name, contact email and MAINNET payout account. Do not infer these from device, GitHub or testnet credentials.
4. Print the complete exact JSON payload and receive the human's explicit yes.
5. Generate requestId once, preserve it, and submit only once. If a network error or 503 occurs, retry with the same requestId. A 201 response is success. A 409 means the repository was already submitted or the email limit was reached; do not try to bypass it.

Never publish, push, post on X or submit the form without the required explicit human confirmation.


## Rechecked 14 November 2026

The live organizer brief still closes submissions at 2026-11-14 23:59 UTC. A public repository and human-published X demo post with attached video or screenshots are required. The application can be sent only after the human confirms the complete payload and contact notice. Solo team details are confirmed. The user asked to retain both X post options as drafts; no social post or application has been sent.
