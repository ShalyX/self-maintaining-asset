# WINNING PROFILE — advisory research assessment

## Assessment status
COMPLETE — confidence MEDIUM.

## Event and judging format
NEAR HACKATHON, November 2026. Primary candidate track: NFTs That Do Something. Secondary candidate: Autonomous Business. Deadline 14 Nov 2026, 23:59 UTC. Checked 13 Nov 2026.

## Evidence reviewed
| Source | Type | What it supports | Limitation |
|---|---|---|---|
| https://nearhackathon.com/ | Official | Public repo + X demo post + form; AI screening then human judging; deadline | Current numeric weights not published |
| https://nearhackathon.com/HACKATHON.md | Official brief | Tracks, submission and technology expectations | Direct page retrieval was inconsistent; cross-checked against event page and official prompt repository |
| https://github.com/nearai/near-hackathon-prompt-examples/tree/main/tracks/10-nfts.md | Official track prompt | NFT-as-account, NEP-171, sub-account direction | Example prompt is guidance, not a compulsory architecture |
| https://github.com/nearai/near-hackathon-prompt-examples/tree/main/tracks/04-business.md | Official track prompt | Autonomous business, treasury and procurement | Secondary track only if genuine |
| https://near.org/blog/meet-the-2023-nearcon-irl-hackathon-winners | Previous winner | Quex won with an AI-powered programmable treasury; autonomous treasury concepts have prior NEAR judging precedent | Different event/year |
| https://near.org/blog/metabuidl-hackathon-winners | Previous winner | Roke.to won grand prize with programmable token streaming/payroll | Different event/year |
| https://github.com/Nebulaz7/Nivry | Current competitor / public repo | Generic NFT-controlled agent treasury and authority handoff research | Repository describes a plan/research; implementation not independently verified |
| https://mintiq-pink.vercel.app | Current competitor / live site | MINTIQ publicly presents NFT treasury, agent hiring, work scoring, reserve rules and payout | Site returned HTTP 200 and public metadata; underlying contract transactions not independently verified |
| https://www.projectmaji.org/maji-solutions | Adjacent product | MajiPay combines water payments, maintenance funding, monitoring and alerts | Not a token-bound asset treasury |
| https://ws.avisers.com/en | Adjacent product | Solar prepaid water kiosk, operator app, revenue visibility and offline operation | Does not publicly advertise an NFT-bound programmable treasury |
| https://play.google.com/store/apps/details?id=com.geotek.watersolutions | Adjacent product | Nigerian borehole monitoring and maintenance alerts | Product feature claims are developer-supplied |
| https://www.hackquest.io/projects/ProofOps | Adjacent product | AI-assisted maintenance evidence, human approval and escrow settlement | Different chain/product context; not confirmed as a NEAR submission |

## Competition landscape
### Direct NEAR competition
- **MINTIQ:** live public testnet demo claimed by its builders. Generic NFTs have AI agents, treasuries, spending rules, reserves, jobs, contributors, approval and payouts. Strong direct overlap with a generic "NFT agent hires technicians" idea.
- **Nivry:** public repository with detailed research/planning for NFT-bound agent authority and keyless treasury. Treat as an emerging concept competitor, not a verified shipped product.

### Adjacent water and asset operations products
- **Project Maji / MajiPay:** water payments, digital balance, maintenance funding, real-time tracking and automated alerts.
- **Water Station:** solar/off-grid prepaid water kiosk, traceable sales, operator app, dashboard and remote reporting.
- **GeoTek Water Solutions:** borehole functionality, availability and maintenance alerts.
- **Borehole Africa:** planned maintenance, emergency callouts, smart metering and AI/automation claims.
- **UpKeep / CMMS precedent:** Kampala water operator case study describes work requests, work orders, preventive maintenance and asset history.

### Competitive conclusion
A generic NFT treasury, generic agent marketplace, generic maintenance app, or water-payment dashboard is not differentiated.
The defensible territory to test is a **physical-asset operating identity**: an asset-specific account and maintenance record whose protected reserve, approved service workflow, and service history remain associated with that asset through operator handover. The AI agent is a constrained maintenance coordinator, not an autonomous unrestricted spender or generic marketplace.

## Likely winning dynamics
- Engineering-led: HIGH. AI judges check reproducibility and README accuracy; on-chain ownership, custody and reserve invariants must be demonstrated.
- Product-led: HIGH. The product must solve a concrete asset-operator workflow, not showcase a wallet.
- Excitement-led: POSSIBLE. A physical asset that receives funds, protects a repair reserve, pays a verified technician and retains its history after transfer creates a clear visual payoff.
- Impact-led: HIGH. Water-point uptime and maintenance financing have credible social/economic significance.
- Design/story-led: HIGH. The asset lifecycle can be explained with a before/after operational story.
- Sponsor/track-led: HIGH. NEP-171, sub-accounts and NEAR smart contracts are central to the proposed experience.

Primary dynamics: product + engineering + originality.
Secondary dynamics: impact + design/story.
Confidence: MEDIUM. Current official weights and track-specific winners are unavailable while the event is still running.

## Excitement opportunity
Status: POSSIBLE.
A physical asset that owns an operating account is immediately understandable and more tangible than a generic agent wallet. The demo payoff must show the maintenance reserve being protected and used, followed by an ownership handover where the same asset account and history remain intact. Do not imply legal title to the physical equipment transfers unless supported by off-chain legal arrangements.

## Ideation recommendation
STANDARD_PLUS_EXCITEMENT_EXPLORATION.
The supplemental concept was selected by the user on 13 November 2026. It remains subject to VALIDATION. No excitement score or bonus is applied.

## Unknowns
- No direct operator interview yet.
- No proof that small Nigerian water operators want or can use an on-chain treasury.
- No fiat/mobile-money on-ramp selected.
- MINTIQ's live testnet contract behavior and full product completeness have not been independently tested.
