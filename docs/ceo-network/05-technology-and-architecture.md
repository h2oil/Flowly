# 05 — Technology & Architecture

## Stack

| Layer | Choice | Why |
|---|---|---|
| Mobile apps | **Expo (React Native) + TypeScript**, one codebase for iOS and Android | CEOs live on their phones. One team ships both platforms. Over-the-air updates for fast fixes |
| Web | **Next.js**: member web app (secondary), marketing site, application form, **admin console** | Admin and verification work happens on desktop. Server rendering for the marketing pages' search ranking |
| Backend | **Postgres** with row-level security (RLS) + edge/serverless functions. Managed with **Supabase** (if a UAE or nearby region is available) or AWS RDS plus a small Node/TypeScript API on AWS **me-central-1 (UAE)** | RLS gives strong per-member data isolation. A UAE region supports the PDPL and trust story. The team already has Supabase experience from the Flowly plan |
| Auth | **LinkedIn OpenID Connect**, email magic link, mobile OTP; Sign in with Apple (Apple's guideline 4.8 requires a privacy-focused login option whenever third-party login is offered on iOS, and Sign in with Apple meets it) | See below |
| Identity / KYC | Vendor with liveness and document checks (Sumsub, Onfido or Veriff), plus **UAE Pass** where eligible | Covers Route B applicants who don't use LinkedIn |
| AI | LLM API (e.g. Claude) for CV parsing, profile drafting, intro reason lines and moderation triage; an embedding model for matching | A large quality gain for a small cost. Every AI output is reviewed or templated |
| Messaging | Postgres-backed chat with realtime subscriptions (Supabase Realtime) or Stream Chat | Start simple. Stream is the fallback if moderation and scale needs grow |
| Push & email | Expo Notifications (APNs/FCM), Postmark or SES for email; WhatsApp Business API for host messages | WhatsApp is the region's default channel for concierge contact |
| Files | Private object storage (S3-compatible, UAE region), signed URLs, encrypted at rest | CVs and verification documents |
| Payments (partners only) | Invoicing by bank transfer for large contracts; Stripe (available in the UAE) for smaller advisor listings | Members never pay, so there are no in-app purchases and no Apple or Google commission |
| Analytics | PostHog (self-hosted or EU/UAE) with privacy-first events | Product analytics without selling data |
| Infra | Vercel (web) + AWS UAE region (data), Terraform, GitHub Actions CI | The repo already deploys to Vercel |

## LinkedIn integration: what is possible
- **Sign In with LinkedIn using OpenID Connect** (self-serve product) gives `sub` (member ID), `name`, `given_name`, `family_name`, `picture`, `email`, `email_verified` and `locale`.
- It **does not** give job title, company, work history or connections. The old open profile APIs are restricted to approved partners.
- **We will not scrape LinkedIn**, which breaks its terms and creates legal risk. The member pastes their public profile URL and a reviewer checks it.
- **Partner programmes worth applying for later:** *Verified on LinkedIn* (identity and workplace verification signals) and the Marketing/Community Management APIs for company-page data. Both are gated, so the plan must work without them.
- **Implementation:** Supabase Auth has a built-in `linkedin_oidc` provider. With a custom backend, use a standard OIDC library. Store the LinkedIn `sub` to prevent duplicate accounts.

## CV parsing pipeline (Route B)
1. Upload PDF or DOCX, maximum 10 MB, to private storage. Run a virus scan.
2. Extract text. Reuse the document importer logic from the Flowly web app (`apps/desktop-ui` handles `.docx` and `.pdf` text extraction in the browser), which is the one reusable piece of the old code.
3. Send the text to the LLM with a strict JSON schema: current role, company, start date, previous roles, sectors, education, languages, leadership signals.
4. Prefill the profile. The applicant confirms and edits.
5. The reviewer sees the parsed data **next to** the original CV. Delete the CV **within 30 days of the decision** unless the member chooses to keep it.

## Core data model (simplified)

```
member(id, auth_user_id, status[applicant|approved|paused|alumni|removed],
       full_name, photo_url, title, member_type, city_id, sector_id, stage,
       size_band_private, bio, focus, languages[], linkedin_sub, linkedin_url,
       trust_score, founding_member, created_at)
company(id, name, website, domain, registry_source, registry_id, verified_at)
membership_application(id, member_id, route[linkedin|cv|experience|invite],
       cv_file_id, parsed_json, signals_json, reviewer_id, decision, notes)
verification_signal(id, application_id, kind, result, weight, evidence_ref)
goal(id, label) / member_goal(member_id, goal_id, kind[seeking|offering], visibility)
intro(id, week, member_a, member_b, score, reason, a_response, b_response,
      curated_by, outcome_rating_a, outcome_rating_b)
conversation / message (members only, E2E optional later)
circle(id, name, kind, host_member_id, city_id) / circle_member
event(id, city_id, partner_id, title, venue, capacity, starts_at)
       / rsvp(event_id, member_id, status, checked_in_at)
ask_offer(id, member_id, kind, body, status, moderation_state)
invitation(id, inviter_id, email, code, redeemed_by, quarter)
partner(id, type[city|event|advisor|capital|search], contract_ref, tier)
advisor_listing / capital_seat / proposal_request (member-initiated only)
audit_log(actor, action, subject, at)   -- every admin access to member data
```

**Row-level security essentials:** a member reads only approved members' *public* fields. Private fields such as goals and size band are readable only by self, by mutual-intro counterparts when the member has shared them, and by staff. **Partners have no direct database access at all**; they use separate partner-portal views of aggregates only.

## Security baseline
- SSO plus mandatory 2-factor authentication for all staff. Least-privilege admin roles (reviewer, host, events, super-admin), with every staff view of personal data recorded in the audit log.
- Encryption in transit (TLS 1.2+) and at rest. Separate storage bucket for KYC documents with short retention.
- Mobile: certificate pinning, jailbreak and root detection warnings, screenshot *warning* on sensitive screens (a full block isn't possible on iOS), biometric app lock option.
- Annual penetration test before launch and yearly after. Bug bounty after about 1,000 members.
- Backups with point-in-time recovery in the same UAE region. Disaster recovery plan documented.

## What happens to the current Flowly code
- The Rust alignment engine, WASM build and Tauri Windows shell (`crates/`, `apps/desktop-shell`, `apps/align-harness`) have **no role** in a networking app. Archive them by tagging the last teleprompter commit (`flowly-teleprompter-final`) and removing them from `main` when the new app is scaffolded.
- **Reusable:** the browser document importer (DOCX/PDF/TXT → text) in `apps/desktop-ui/src` for CV upload pre-parsing; the Vercel deployment setup; the Playwright e2e test-rig pattern.
- **New repo layout (proposed):**
  ```
  apps/mobile        Expo app (iOS/Android)
  apps/web           Next.js — marketing, apply, member web, admin console
  packages/ui        shared design system (tokens, RTL-ready components)
  packages/core      shared types, validation (zod), matching logic
  supabase/ or api/  migrations, RLS policies, edge functions
  docs/ceo-network   this plan
  ```
