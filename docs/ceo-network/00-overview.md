# CEO Network: Plan Overview

> **Pivot notice.** This plan replaces the Flowly teleprompter. The product is now a **verified, invite-only networking app for CEOs from every sector**. It is **free for members**, funded by partners, and **launches in Dubai** before rolling out city by city.
>
> Working name in this plan: **Majlis**. A *majlis* is the Gulf tradition of a sitting room where leaders meet. The name is a placeholder until trademark clearance; alternatives are in [01](01-product-and-membership.md#naming).

## Executive summary

CEOs are the hardest people in any city to reach, and they have the fewest good places to meet each other. LinkedIn is full of sellers, recruiters and noise. Paid peer groups such as YPO, EO and Vistage are expensive, slow to join and built around fixed forums. WhatsApp groups don't scale and have no vetting.

Majlis is a **mobile-first, verified-only network** where every member is a confirmed CEO, founder-CEO, managing director, country head or division CEO. Members get:

1. **Curated introductions.** A few high-quality, double-opt-in intros a week, matched on goals rather than titles.
2. **Circles.** Small groups by sector, company stage and topic, such as "Family business succession" or "Scaling into KSA".
3. **Private events.** Invite-only dinners, roundtables and an annual summit, held under the Chatham House Rule.
4. **Asks and Offers.** A trusted board for members to ask for help ("Need a regulator contact in ADGM") or offer it.

**Members never pay.** Revenue comes from carefully controlled partner access: event sponsors, a vetted Trusted Advisor marketplace, Capital Partner seats for investors, board and NED placement, and sponsored CEO insight reports. The guiding rule is that **the member is never the product**. No member data is sold, nobody gets cold pitches, and every partner touchpoint is opt-in.

## Decisions at a glance

| Decision | Choice | Why |
|---|---|---|
| Who gets in | Verified top executives across all sectors: CEO, founder-CEO, MD, GM/country head, division or business-unit CEO | "All divisions" means every industry *and* divisional leaders inside large groups. This is a much bigger pool than founders alone, and the strongest pull in the Gulf |
| Entry routes | (A) **Sign in with LinkedIn**, or (B) **member request** by CV upload or a written experience summary. Both routes then go through layered verification | LinkedIn's sign-in only proves *identity* (name, email, photo). It does **not** share job titles, so we verify the role separately (see [01](01-product-and-membership.md#verification)) |
| Verification | Identity check + work-email domain check + company registry check + AI-assisted CV parsing + **human review** + optional vouching by an existing member | Trust is the product. One fake "CEO" who turns out to be a salesperson costs more than slow onboarding |
| Price to members | **Free, permanently**, with no paid tier for CEOs | Removes the main objection to YPO and EO. Free plus verified also lets the network grow faster than paid rivals |
| Monetisation | Partners pay to reach members **through hosted experiences, never through data**: sponsorships, Trusted Advisors, Capital Partners, placements, insights | Partners include banks, wealth managers, law firms and free zones. They value CEO access highly, and members accept them when the rules protect members |
| Launch city | **Dubai.** Founding cohort of about 100 hand-picked CEOs, then invite-only growth to about 600 in year 1 | High CEO density, international mix, an event culture and strong sponsor budgets |
| Expansion | Abu Dhabi → Riyadh → Doha, then London, Singapore and Mumbai, using a repeatable City Launch Kit | Gulf cities first share partners and members. The international hubs follow the UAE's trade corridors |
| Stack | Expo (React Native) apps for iOS and Android, a Next.js web app and admin console, Postgres with row-level security, hosted in a UAE cloud region | One codebase for both phone platforms, plus strict data isolation and UAE data residency for trust |
| Existing code | The Rust/Tauri teleprompter is **archived, not reused** | No overlap with a social or mobile product (see [05](05-technology-and-architecture.md#what-happens-to-the-current-flowly-code)) |

## Plan documents

| # | Document | Covers |
|---|---|---|
| 01 | [Product & membership](01-product-and-membership.md) | Concept, target members, eligibility, sign-up and verification flows, naming |
| 02 | [Features & UX](02-features-and-ux.md) | MVP feature set, key screens, matching engine, member rules |
| 03 | [Business model](03-business-model.md) | Why free works, the seven revenue streams, pricing, 3-year projection, funding need |
| 04 | [Go-to-market: Dubai & expansion](04-go-to-market.md) | Founding cohort, launch sequence, partners, City Launch Kit, expansion order |
| 05 | [Technology & architecture](05-technology-and-architecture.md) | Stack, LinkedIn integration, CV parsing, data model, security |
| 06 | [Trust, safety & legal](06-trust-safety-legal.md) | UAE PDPL and DIFC data law, regulated activities to avoid, moderation, company setup |
| 07 | [Roadmap, KPIs & risks](07-roadmap-kpis-risks.md) | Phased timeline, team, metrics, risk register |

## MVP in one paragraph

Verified onboarding through LinkedIn sign-in or a CV/experience application, each reviewed by a person. A rich CEO profile. Weekly curated introductions with double opt-in. One-to-one messaging. Circles. Event invitations with RSVP and check-in. Asks and Offers. An admin console for verification, matching overrides, events and partner management. **Deferred past MVP:** Trusted Advisor marketplace UI, Capital Partner seats, deal rooms, insights reports, multi-city.
