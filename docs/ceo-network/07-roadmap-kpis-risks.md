# 07 — Roadmap, KPIs & Risks

## Phased roadmap

| Phase | Months | Product | Business | Exit criteria |
|---|---|---|---|---|
| **0. Foundation & concierge MVP** | 0–2 | Landing page + application form (web), Airtable/Notion back-office, manual intros by WhatsApp/email | Entity setup, brand and trademark, 50 Founding Members, first 1–2 partners, seed round opens | ≥ 60% intro acceptance; ≥ 70% "would recommend"; 1 signed partner |
| **1. Build & private beta** | 2–5 | Expo app + admin: onboarding (LinkedIn/CV), verification queue, profiles, weekly intros, chat, events, Circles | 100 Founding Members, 4 events, seed closed, Partnerships Lead and 2 hosts hired | Crash-free ≥ 99.5%; median verification < 48h; weekly active ≥ 50% |
| **2. Dubai launch** | 6–9 | Asks & Offers, directory, Arabic UI polish, partner credits on events, analytics | "Dubai 100" launch evening, PR, CEO Pulse #1, 2 City Partners | 300 members; ≥ 40% monthly active; event NPS ≥ 60 |
| **3. Monetise depth** | 9–12 | Trusted Advisor marketplace, Capital Partner seats, partner portal (aggregate dashboards), AI meeting briefs | 25 advisors, 8 capital seats, 8+ sponsored events | 600 members; AED ~1.7M revenue run-rate |
| **4. Multi-city** | 12–18 | City Passport, multi-city data model, KSA compliance, board/NED marketplace | Abu Dhabi (m12), Riyadh (m15) launches via City Launch Kit; Series A prep | 2,500 members across 3 cities; ≥ 70% partner renewal |
| **5. International** | 18–36 | Deal rooms, learning-to-rank matching, audio office hours, CEO Pulse at scale | Doha, London, Singapore | 7,000 members; operating profit |

## Team plan

| Role | When | Notes |
|---|---|---|
| Founder / CEO | Day 0 | Network-led founder; the face of the community |
| CTO / lead engineer | Day 0 | Full-stack TypeScript, mobile experience |
| Product designer | Month 1 | Premium, bilingual design system |
| 2 engineers (mobile + backend) | Month 2 | |
| Member Success hosts × 2 | Month 1–3 | Verification, curation, concierge; hospitality or private-banking background |
| Partnerships Lead | Month 2 | Sells City Partner and event packages |
| Events manager | Month 4 | Owns event quality |
| City Leads | One per new city, T−3 months | Commission on local partner revenue |
| Advisors | Day 0 | 3–5 respected Dubai CEOs (equity), 1 data-protection lawyer, 1 community-business operator |

## KPIs (north star and supporting)

**North star:** **valuable meetings per month**, meaning intros that both members accept *and* later rate 4 or 5 out of 5.

| Area | Metric | Target (steady state) |
|---|---|---|
| Acquisition | Applications per week; approval rate; median verification time | Rising; 50–70%; < 48h |
| Activation | % of new members who accept at least 1 intro in their first 14 days | ≥ 70% |
| Engagement | Monthly active members; intro acceptance rate; event attendance rate | ≥ 40%; ≥ 50%; ≥ 75% of RSVPs |
| Quality | Intro rating (1–5); member NPS; reports per 1,000 members | ≥ 4.0; ≥ 50; falling |
| Network density | Average mutual connections per member within city | Rising |
| Monetisation | Partner revenue; partner renewal; revenue per member | Plan in [03](03-business-model.md); ≥ 70%; ≥ AED 5k by year 3 |
| Trust | Verified-fake incidents; data incidents | 0; 0 |

## Risk register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | **CEOs don't make time**: low engagement after signup | Medium | High | Value in under 2 minutes a week; human hosts; events as the anchor; pause inactive members' intros rather than spamming them |
| 2 | **Fake or "soft" CEOs get in** (salespeople, consultants) and quality collapses | Medium | Very high | Layered verification plus a human approval for every member; vouching; fast removal; strict thresholds |
| 3 | **Partners feel like ads** and members leave | Medium | High | Hosted-experience model only; frequency caps; mute option; member advisory council reviews partner formats |
| 4 | **Data breach or leak of member info** | Low | Very high | UAE-region hosting, RLS, audit logs, minimal data retention, pen tests, cyber insurance |
| 5 | **LinkedIn API changes or restrictions** | Medium | Medium | LinkedIn is only one identity option; CV, OTP and KYC routes exist; no dependence on profile APIs |
| 6 | **Regulatory issues** (investment arranging, recruitment licensing, data transfer) | Medium | High | Intro-only and flat-fee model; licensed partners for placements; legal opinions before each new stream |
| 7 | **Competitors copy** (YPO/EO digital push, LinkedIn premium groups, local clubs) | Medium | Medium | Free plus verified plus Gulf-native; density head start; founding member loyalty; City Passport network effect |
| 8 | **Slow partner sales** delay revenue | Medium | High | Sign founding partners in Phase 0; prove with manual events before the app; keep the cost base lean until 2 partners are signed |
| 9 | **City expansion dilutes quality** | Medium | High | City go/no-go gates; a local City Lead each time; same verification standards everywhere |
| 10 | **Key person dependency** (founder network) | High (early) | Medium | Institutionalise curation in the admin tool; advisor bench; City Leads |

## Immediate next steps (next 30 days)
1. **Decide name and entity.** Run trademark searches and pick a free zone. The DIFC Innovation Hub is recommended for credibility.
2. **Recruit 10 anchor Founding Members** personally, ideally across 10 different sectors.
3. **Hold 10 partner discovery calls** (banks, law firms, free zones) to test pricing in [03](03-business-model.md).
4. **Ship the landing page and application form** (Next.js on the existing Vercel project) with LinkedIn sign-in and CV upload, backed by a manual review sheet.
5. **Run the first concierge intro week and first dinner** to collect acceptance rates and testimonials.
6. **Prepare the seed deck**, built from this plan, the early metrics and partner letters of intent.
