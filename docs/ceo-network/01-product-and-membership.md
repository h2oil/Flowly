# 01 — Product & Membership

## The problem

| Who | Pain today |
|---|---|
| CEOs new to Dubai | Thousands of CEOs relocate to Dubai every year. They don't know their peers, and the only way in is through slow, personal referrals |
| Established local CEOs | Inboxes and LinkedIn are flooded with sales pitches, and there is no quiet, peer-only space |
| Division and regional CEOs in large groups | Inside their company they have no true peers. Outside it, they don't fit founder communities |
| Everyone | Paid peer groups (YPO, EO, Vistage) charge about USD 3k–15k a year, take months to join and lock members into one fixed forum |

## The promise

> **"Every person here is a verified CEO. Nobody is selling to you. It's free."**

## Who can join (eligibility)

"All divisions" is read in two ways: **every industry sector**, and **every kind of top executive**, including leaders of divisions inside large groups.

| Member type | Eligible titles (examples) | Minimum company signal (Dubai launch) |
|---|---|---|
| Founder-CEO | CEO, Founder & CEO, Co-founder & CEO | Operating company with **≥ 10 staff**, or **≥ USD 1M revenue**, or **≥ USD 2M raised** |
| Appointed CEO / MD | CEO, Managing Director, President | Company with ≥ 25 staff |
| Regional / country head | GM, Country Manager, Regional CEO, Head of MENA | Multinational subsidiary with ≥ 25 staff in the region |
| Division / business-unit CEO | Divisional CEO, BU President, CEO of a group subsidiary | The division alone meets the "appointed CEO" bar |
| Family business principal | Chairman, Vice-Chairman, Managing Partner with executive role | Family group with operating businesses |
| Public-sector chief executive (invited only) | DG, CEO of a government entity or free zone | By invitation only |

Excluded, so the room stays a room of peers: other C-suite roles (CFO, COO, CMO), solo consultants, and recruiters, brokers or salespeople under a "CEO" title. These people can still reach members through the partner programmes in [03](03-business-model.md).

**Thresholds are configurable per city.** Smaller markets may lower them. Over time the network may add a separate **"Next CEO"** track for COOs and CEO successors as a growth lever.

### Segmentation for matching and Circles
- **Sector** (about 20 top-level sectors): Financial services, Real estate & construction, Energy & utilities, Tech & software, Healthcare, Retail & consumer, Hospitality & F&B, Logistics & trade, Manufacturing, Professional services, Media, Education, Family conglomerates, Government & semi-government, and others.
- **Company stage:** Startup, Scale-up, Established SME, Large enterprise / multinational division.
- **Company size band:** Headcount and revenue bands, stored but **never shown publicly**. Members see a band only if the other member opts in.
- **Goals (the matching fuel):** Hiring a leadership team, raising capital, entering KSA, finding distributors, succession, M&A, board seats, learning AI adoption, and more.

## Sign-up flows

There are two routes in. Both end at the same verification pipeline and the same human review.

```
          ┌────────────────────────┐        ┌──────────────────────────────┐
Route A → │ Sign in with LinkedIn  │        │ Request membership           │ ← Route B
          │ (OpenID Connect)       │        │ CV upload (PDF/DOCX) OR      │
          │ → name, email, photo,  │        │ experience form + references │
          │   verified identity    │        │                              │
          └──────────┬─────────────┘        └──────────────┬───────────────┘
                     │                                     │
                     ▼                                     ▼
          ┌──────────────────────────────────────────────────────────────┐
          │ Role & company step: title, company, website, work email,    │
          │ trade licence no. (optional), LinkedIn profile URL           │
          └──────────────────────────┬───────────────────────────────────┘
                                     ▼
          ┌──────────────────────────────────────────────────────────────┐
          │ Automated checks (see Verification) → trust score 0–100      │
          └──────────────────────────┬───────────────────────────────────┘
                                     ▼
          ┌──────────────────────────────────────────────────────────────┐
          │ Human review (target < 48h)  → Approve / Ask for more / Deny │
          │ Optional: 15-min video welcome call for borderline cases     │
          └──────────────────────────┬───────────────────────────────────┘
                                     ▼
                    Welcome → profile completion → first 3 intros
```

### Route A: LinkedIn sign-in (the fastest path, about 3 minutes)
1. The user taps **Continue with LinkedIn**. We use LinkedIn's "Sign In with LinkedIn using OpenID Connect" product, which returns name, email, profile photo and a LinkedIn member ID.
2. **Important limit:** LinkedIn's standard sign-in API does **not** return job title, company or work history, and scraping LinkedIn breaks its terms. LinkedIn therefore proves the person is real, but it does **not** prove they are a CEO.
3. The user confirms title and company and pastes their public LinkedIn profile URL. A reviewer checks it by hand. Optionally the user adds a work email for a domain check.
4. **Later upgrade:** Apply to LinkedIn's **"Verified on LinkedIn"** partner programme. If accepted, it can confirm identity and workplace verification status directly. This is a partner-gated programme, so plan without it and treat it as a bonus.

### Route B: member request (CV or experience)
For CEOs who aren't active on LinkedIn, which is common among family-business principals and government-linked leaders:
1. Enter name, mobile (OTP verified) and email.
2. **Upload a CV** (PDF or DOCX), **or** fill a short experience form: current role, company, years in role, two previous roles, and a 150-word "what I'm building".
3. Optionally name **two referees** or **one existing member** to vouch for them.
4. An AI parser pulls out structured data (current title, company, tenure, sectors, previous roles) and fills in the profile, which the applicant then edits. The CV file is kept only until review ends and then deleted, unless the member chooses to keep it (see [06](06-trust-safety-legal.md)).

### Route C: invitation (the growth engine)
Each member gets **3 invitations per quarter**. An invited applicant gets a **+25 trust score** boost and priority review. They still go through verification. The person who invited them is notified, and repeated bad invites reduce that member's invitation allowance.

## Verification

Trust score signals, combined into a score from 0 to 100:

| Signal | How | Weight |
|---|---|---|
| Real person | LinkedIn OIDC login, **or** mobile OTP plus selfie liveness check (vendor such as Onfido, Sumsub or UAE Pass where available) | High |
| Works where they say | Email OTP to a **work domain** that matches the company website; free mail domains get no points | High |
| Company is real | Lookup in public registers: DED/Invest in Dubai, DIFC public register, DMCC, ADGM registry, MoE; later Companies House (UK), ACRA (SG) and Wathq (KSA) | High |
| Role is senior | Title on LinkedIn URL (reviewer check), CV tenure, company website "leadership" page, press mentions | Medium |
| Company size meets threshold | Headcount from LinkedIn company page or the applicant's own statement, plus funding data (Crunchbase or MAGNiTT for MENA) | Medium |
| Social proof | Vouch from an existing member, or a referee reply | Medium–High |
| Red flags | Title says CEO but the company is a one-person consultancy; sales or recruiting keywords; mismatched domain; duplicate applicant | Negative |

**Decision rules (configurable)**
- Score ≥ 80 with no red flags: **fast-track**. A reviewer approves in one click.
- Score 50–79: **standard review**. The reviewer may ask for one more proof, such as a trade licence page or a company letter.
- Score < 50: **ask for more** or **politely decline**, with a "reapply in 6 months" option.
- **Every approval has a human in the loop.** AI only prepares the case.

**Re-verification:** a yearly prompt to confirm role ("Still CEO of X?"). Role changes trigger a short review. A member who leaves a CEO role keeps an **Alumni** badge for 12 months and can then move to the Next CEO track.

**Badges shown on profiles:** ✓ Verified CEO · Company verified · Vouched by N members · Founding Member.

## Naming

Chosen name: **Flowly**, with a cursive single-stroke wordmark and a gold cursive "f" app icon in `docs/brand/`. Fallbacks if clearance fails: **Majlis**, **Peerage**, **The CEO Table**, **Diwan**. Before committing:
- Run trademark searches with the UAE Ministry of Economy, WIPO Global Brand Database, UK IPO and USPTO for classes 9, 35, 41 and 45.
- Check App Store and Google Play name availability, plus the `.com` and `.ae` domains and social handles.
