# 08 — App Specification (screens, flows, functions)

This document is the build spec for the member app (iOS and Android) and the parts of the web app that mirror it. It details every function: registration request, profile, chat, connections, events, locations and the rest. The labels **MVP / v1.1 / v2** show when each item ships (see the [release plan](#release-plan)).

---

## 1. App map

**Five bottom tabs** plus a global header with a city switcher, search and notifications.

```
┌──────────────────────────────────────────────────────────────┐
│  Header:  [Dubai ▾]            🔍 Search          🔔 (3)      │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│                         (tab content)                        │
│                                                              │
├──────────┬──────────┬──────────┬──────────┬──────────────────┤
│  🏠 Home │ 👥 Network│ 💬 Chat  │ 📅 Events │ 👤 Me            │
└──────────┴──────────┴──────────┴──────────┴──────────────────┘
```

| Tab | What lives here |
|---|---|
| **Home** | This Week: 3 intros, next event, Asks you can help with, Circle highlights, city news from the host |
| **Network** | Directory and search, connections, Circles, Asks & Offers, Places and map, Trusted Advisors (v1.1) |
| **Chat** | All conversations: 1-to-1, Circle groups, event chats, the City Lounge (general chat), concierge |
| **Events** | Upcoming events (list, calendar, map), my tickets, member-hosted meetups, past events and recaps |
| **Me** | My profile, digital business card, invitations, saved items, travel (City Passport), settings and privacy |

**Floating "+" button** (Home and Network): Post an Ask · Post an Offer · Host a meetup · Invite a CEO · Share my card.

---

## 2. Registration request (application flow)

### 2.1 Screens: applicant side **(MVP)**

| # | Screen | Content and fields | Rules |
|---|---|---|---|
| R1 | **Welcome** | Brand video loop · "Continue with LinkedIn" · "Apply with CV" · "Apply with experience" · "I have an invitation code" · Sign in | Arabic/English toggle at the top |
| R2 | **Invitation code** (optional) | 8-character code or deep link from an invite | Shows "Invited by *Name*" with photo; +25 trust score |
| R3 | **Identity** | *LinkedIn path:* OIDC consent pre-fills name, email and photo. *Other paths:* full name, email, mobile number with country code | Mobile **OTP required** for everyone; email verified by magic link |
| R4 | **Your role** | Job title (picklist plus free text) · Member type (Founder-CEO / CEO-MD / Regional head / Division CEO / Family principal) · Company name (autocomplete from our company table) · Company website · Work email (optional but encouraged) | A work-email OTP adds trust points; free mail domains are flagged |
| R5 | **Your company** | Sector (one of 20) · Sub-sector · Stage · Headcount band · Revenue band (private) · HQ city · Trade licence number (optional) · Year founded | Revenue band is **never shown** to other members without opt-in |
| R6 | **Your experience** | *CV path:* upload PDF/DOCX (10 MB max) and AI fills in the fields. *Experience path:* current role start date, 2 previous roles (title, company, years), short "what I'm building" (150 words) · LinkedIn profile URL (optional on these paths) | Applicant reviews and edits the AI-filled fields |
| R7 | **References** (optional) | Vouching member (search members) **or** 2 referees (name, title, email) | Referees get a one-click "Yes, I know them and their role" email |
| R8 | **Why join** | Choose up to 3 goals: hire leaders, raise capital, expand to KSA, find partners or distributors, peer advice, board roles, learn (AI, succession…) + "Can help others with" (up to 5 tags) | Feeds matching from day one |
| R9 | **Agreements** | Member Code · Terms · Privacy notice · optional consents (CEO Pulse surveys, partner event invitations) | Must scroll the Member Code to accept |
| R10 | **Submitted** | "Your application is with *Host Name*." Status timeline. Expected time: under 48 hours | Push and email confirmation |

### 2.2 Application statuses

```
Draft → Submitted → In review → (Needs more info ↺) → Approved → Onboarding → Active
                                     └──────────────→ Declined (reapply after 6 months)
```

- **Needs more info:** the applicant gets a push notification and sees a checklist, for example *"Upload trade licence first page"* or *"Book a 15-minute welcome call"* (in-app calendar slots).
- **Draft autosave:** the applicant can leave and resume. A reminder goes out after 24 hours and again after 72 hours.
- **Declined:** a respectful message, no specific reason given if fraud is suspected, plus the reapply date.

### 2.3 Approval and onboarding **(MVP)**
1. **Approval moment:** a full-screen welcome with a "Founding Member" badge if they qualify, and a personal note from their host.
2. **Profile polish (2 minutes):** the AI-drafted bio is shown for editing; photo crop; choose directory visibility.
3. **Pick Circles:** 3–6 suggested Circles to join.
4. **Notification setup:** weekly digest day and time (default Sunday 08:00), quiet hours (default 22:00–07:00).
5. **First intros:** 3 intros delivered immediately rather than waiting for the weekly cycle.
6. **Invite peers:** shows the 3 invitations available this quarter (skippable).

### 2.4 Reviewer side (admin console) **(MVP)**
- Queue sorted by trust score and waiting time, with SLA timers (amber at 24 hours, red at 48).
- Case view: applicant data **next to** evidence (CV, LinkedIn URL preview link, registry result, email-domain match, referee replies), a trust score breakdown and red flags.
- Actions: Approve · Approve as Founding · Ask for more info (templates) · Book call · Decline (reason codes) · Escalate.
- Two reviewers required for declines of invited applicants and for any case flagged as fraud.

---

## 3. Profile

### 3.1 Profile sections

| Section | Fields | Default visibility |
|---|---|---|
| **Header** | Photo, full name, title, company (linked company page), city, badges (✓ Verified CEO · Company verified · Founding Member · Vouched by N) | All members |
| **About** | Bio (500 characters), "Currently focused on" (140), languages, nationality (optional) | All members |
| **Company** | Logo, sector, stage, HQ, website, headcount band | All members |
| **Size & money** | Revenue band, funding stage, raising now? | **Private**; shared only with mutual connections on opt-in |
| **Goals** | Looking for (up to 3) | Matching engine + mutual connections only |
| **Can help with** | Up to 5 expertise tags | All members |
| **Experience** | Previous roles (from CV or manual), boards held, education | All members (each item can be hidden) |
| **Interests** | Up to 8 interest tags: golf, padel, sailing, art, philanthropy, F1… | All members (great for casual meetups) |
| **Circles & events** | Circles joined, events attended (last 12 months) | All members (can be hidden) |
| **Mutuals** | "You both know…" and shared Circles | Viewer-specific |
| **Contact** | Mobile, email, PA/EA contact, calendar link | **Only after connecting**, and per-field toggle |

### 3.2 Profile functions
- **Edit profile**, with a completeness meter. Profiles under 70% complete get fewer intros, which is explained to the member.
- **PA / Executive Assistant delegate (v1.1):** the CEO can add an EA with **limited access** (scheduling and event RSVPs only, no chat reading). Common and important for this audience.
- **Profile actions on someone else's profile:** Connect (with a note) · Request intro via host · Message (if connected) · Save · Share to a member · Report · Block.
- **Who viewed me (opt-in both ways, v1.1):** visible only if the member also lets others see their own views.
- **Status line:** "Open to coffee this week", "Travelling to Riyadh 12–14 Dec", "Heads-down, intros paused". This controls matching too.
- **Digital business card (MVP):** a QR code in Me → My card. Scanning it at an event (member or not) opens a profile preview; for members it opens a fast "Connect" sheet. Includes an Apple/Google Wallet pass.
- **Company page:** auto-created. Lists all verified members from that company group, such as several division CEOs, plus company details.

---

## 4. Connections & introductions

| Function | How it works | Limits | Release |
|---|---|---|---|
| **Weekly intros** | 3 curated matches with a reason line; Meet / Not now | Double opt-in opens chat | MVP |
| **Connect request** | From a profile, with a required note (300 characters) explaining why | **5 requests per week** to prevent spam; recipient sees note and mutuals; Accept / Ignore (sender isn't told) | MVP |
| **Host intro** | "Ask my host to introduce me to *X*". The host writes a warm double-opt-in intro | 2 per month | MVP |
| **Event connections** | After an event: "Who did you meet?" Tick attendees to send instant mutual-connect requests | Attendees of that event only | MVP |
| **Card scan** | QR at events: instant connect | — | MVP |
| **Intro feedback** | 7 days after a match: "Did you meet? Rate 1–5" | Feeds matching | MVP |
| **Pause intros** | Snooze 1/2/4 weeks | — | MVP |

---

## 5. Chat

### 5.1 Conversation types

| Type | Who can join | Purpose | Release |
|---|---|---|---|
| **1-to-1 chat** | Two connected members, or a mutual intro | Private conversation | MVP |
| **Small group chat** | Started by a member with up to 12 of their connections | Deal or partnership discussions, trip planning | MVP |
| **Circle chat** | Circle members (8–40) | Topic or sector discussion with threads; host moderates | MVP |
| **City Lounge (general chat)** | **All verified members of a city** | Open general conversation: city news, quick questions, "anyone at the summit today?" | MVP |
| **Global Lounge** | All members, all cities | Cross-city conversation once multi-city | v1.1 |
| **Event chat** | RSVP'd attendees | Opens 72 hours before, stays 7 days after; logistics, "where's the parking", follow-ups | MVP |
| **Concierge chat** | Member ↔ their Member Success host | Help, intro requests, feedback; also bridged to WhatsApp if the member prefers | MVP |
| **Advisor proposal thread** | Member ↔ Trusted Advisor (**member starts it**) | Proposal requests | v1.1 |
| **Deal room** | Invited members, NDA click-through | Confidential deal or partnership with files and audit log | v2 |

### 5.2 City Lounge: how a general chat stays high quality
- **Channels inside the Lounge:** `#general`, `#ask-the-city` (quick help), `#hiring-talent` (leadership hires only), `#travel-and-visitors`, `#wins` (celebrate members), `#events-chatter`. Hosts can add channels per city.
- **Threads** under every message, so the main stream stays readable.
- **Rules enforced in the product:**
  - Slow mode: 1 post per 10 minutes per member in `#general`.
  - No links in a member's first 7 days.
  - Automatic AI flag for sales language, which sends the post to a moderator before it shows.
  - No direct messages from the Lounge unless connected; a "Connect" button is offered instead.
- **Digest instead of noise:** Lounge notifications are **off by default** and summed up in the weekly digest ("Top 5 Lounge threads this week"). Members can turn on @mentions only.
- **Pinned items:** the host pins the weekly agenda, upcoming events and city announcements.

### 5.3 Chat features

| Feature | Release |
|---|---|
| Text, emoji, reactions, replies, @mentions, threads (groups) | MVP |
| Photos, PDF and document sharing (25 MB max, virus-scanned) | MVP |
| Voice notes, very common in the Gulf (with optional transcript) | MVP |
| **Share a profile** or **share an event** card into chat | MVP |
| **Propose a meeting:** pick 3 slots and a place (Places list) → other side picks one → calendar invite to both | MVP |
| Read receipts and typing indicator (each member can turn them off) | MVP |
| Message search across all chats | MVP |
| Pin and mute conversations; archive | MVP |
| **Auto-translate** Arabic ⇄ English per message (tap to translate) | v1.1 |
| Disappearing messages (24h / 7d) in 1-to-1 and small groups | v1.1 |
| Polls in groups and Circles | v1.1 |
| Audio/video call (1-to-1, via a calling SDK) | v2 |
| End-to-end encryption for 1-to-1 and small groups (Signal-protocol library) | v2 |
| Report message · Block member · Leave group | MVP |
| Screenshot **warning** banner on Circle and deal-room chats (Android can block screenshots; iOS can only detect them) | MVP |

---

## 6. Events

### 6.1 Event types

| Type | Host | Size | Access |
|---|---|---|---|
| **Flagship** (city launch "100", annual summit) | Majlis + City Partners | 100–500 | Invite or apply; all members can apply |
| **Partner dinner / roundtable** | Majlis + a sponsor | 12–40 | Curated invite list (matching picks the right mix), or apply |
| **Circle meetup** | Circle host | 6–20 | Circle members |
| **Member-hosted meetup** | Any member (approved by staff) | 4–12 | Open to members or by invite: "Padel at 7am, DIFC", "Breakfast on scaling into KSA" |
| **Online session** | Majlis / experts | Unlimited | Members; live video plus recording (v1.1) |
| **Experience** | Partner (yacht, desert, F1, art preview) | 10–30 | Apply; lottery if oversubscribed |
| **Travel meetup** (City Passport) | Members visiting a city | 4–15 | Members in that city that week (v1.1) |

### 6.2 Event screens and functions

| Function | Detail | Release |
|---|---|---|
| **Discover** | List · calendar · **map** views; filters by type, date, city, Circle, "spots left" | MVP |
| **Event page** | Cover image, title, host(s), partner credit ("Hosted with ___"), date/time, venue with map pin, dress code, agenda, speakers (members), capacity bar, who's going (members only; each attendee can hide themselves) | MVP |
| **RSVP modes** | *Open* (instant yes) · *Request to attend* (host approves; the request includes "what you'd like to get out of it") · *Invite-only* · *Lottery* | MVP |
| **Waitlist** | Automatic promotion with a 12-hour confirm window | MVP |
| **Plus-one / nominate a CEO** | Optional per event; the guest gets a fast-track application | MVP |
| **Ticket** | QR ticket in app and Apple/Google Wallet; add to calendar | MVP |
| **Reminders** | 24 hours and 2 hours before; directions and parking info | MVP |
| **Check-in** | Host/staff scan the QR in the admin app, or self check-in via geofence at the venue | MVP |
| **Event chat** | See §5.1 | MVP |
| **"Who's here" (live)** | During the event: checked-in attendees with photos, so members can find people in the room | MVP |
| **Post-event** | "Who did you meet?" → connect; rating (1–5); photos gallery; host recap; slides or notes (Chatham House safe) | MVP |
| **Host a meetup** | Member flow: type, title, date, place (from Places or custom), capacity, open or invite → staff approval within 24 hours → published | MVP |
| **Cancellation and no-shows** | Cancel up to 24 hours before; 2 no-shows in 6 months means lower priority for curated events | MVP |
| **Dietary and access needs** | Stored once in profile, shared with the host for events the member RSVPs to | MVP |
| **Online sessions** | Join link, live Q&A, recording library | v1.1 |
| **Paid premium experiences** | Only if partner-funded; members stay free | — |

### 6.3 Admin and partner tools for events
- Event builder (web), guest-list curation helped by matching ("best 30 CEOs for this fintech roundtable"), invite waves, seating plan generator (v1.1), check-in app, attendance report, **partner report** (aggregate: attendance, sector mix, satisfaction, follow-up connections; no personal data without consent).

---

## 7. Locations: cities, Places & presence

### 7.1 City model **(MVP)**
- Every member has a **home city**. Content (Lounge, events, intros, directory) defaults to the city chosen in the header switcher.
- At launch the only city is Dubai, with neighbourhoods as sub-areas: DIFC, Downtown, Business Bay, Dubai Marina / JLT (DMCC), Media City / Internet City, Al Quoz / Al Barsha, Deira, Dubai South / Expo City, Palm Jumeirah.
- **Abu Dhabi** (ADGM / Al Maryah, Saadiyat, Yas) and **Riyadh** (KAFD, Olaya, Diplomatic Quarter) are added in v1.1/v2.

### 7.2 Places (venue directory) **(MVP light → v1.1 full)**
- A curated list of **CEO-friendly meeting places**: hotel lounges, private clubs, quiet restaurants, partner offices with meeting rooms.
- Each Place has a photo, area, map pin, type, "good for" (coffee, lunch, private meeting, dinner), opening hours and **member perks** (partner venues: priority reservations, private room, welcome drink). Perks are a sponsorship product.
- Used in: "Propose a meeting" in chat, event venues, member-hosted meetups.
- **Meeting room booking (v1.1):** reserve a partner meeting room (DIFC, Downtown, DMCC partner spaces) for a 1-to-1 or small group, free for members and paid by the venue partner.
- **Map view:** Places plus upcoming events on one city map.

### 7.3 Presence: "Open to meet nearby" **(v1.1, opt-in, privacy-first)**
- A member can switch on **"I'm around"** for 1–4 hours, choosing a **neighbourhood** (e.g. "DIFC"). We **never share exact GPS**; at most the device suggests the area.
- Other members who also switched it on, or who are connected, see "3 members open to coffee in DIFC now", and can send a quick "Coffee?" request.
- It turns off automatically when the time is up. It is never on by default, and it is hidden from members the person has blocked.

### 7.4 City Passport (travel mode) **(v1.1)**
- Add a trip: city and dates (or import from calendar).
- Before the trip: "12 members in Riyadh are open to meeting visitors" (they opted in), events during the stay, and that city's Places.
- The member appears in the destination city's `#travel-and-visitors` channel and the visitor list (opt-in).
- Intros that week include local members.
- The local host gets notified of high-value visitors and can arrange a travel meetup.

---

## 8. Other functions

| Function | What it does | Release |
|---|---|---|
| **Home: This Week** | 3 intros, next event, "Asks you can help with", Circle highlights, host message, Lounge top threads | MVP |
| **Asks & Offers** | Post an Ask or Offer (280 characters, category, city or global, expiry 14 days); members reply privately; "Solved" marker; reputation for helpful members | MVP |
| **Circles** | Browse and join (open, or by request); Circle home with chat, members, next meetup, files, pinned agenda; each Circle has a member host | MVP |
| **Directory and search** | Search by name, company, sector, stage, city, "can help with", interests; saved searches; members hidden from the directory never appear | MVP |
| **Notifications centre** | All alerts in one place; per-category controls; quiet hours; weekly digest email and push | MVP |
| **Invitations** | 3 per quarter; track status (sent, applied, approved); share link or code via WhatsApp, SMS or email | MVP |
| **Saved** | Saved profiles, events, Asks and Places | MVP |
| **Concierge** | Talk to your host; request intros; event help; feedback | MVP |
| **Calendar sync** | Google / Outlook / Apple: event RSVPs and confirmed meetings added automatically; busy-time awareness for meeting proposals (v1.1) | MVP (one-way) |
| **Knowledge library** | CEO Pulse reports, session recordings, member-written playbooks (e.g. "Setting up in KSA: what we learned") | v1.1 |
| **Trusted Advisors** | Vetted firms directory; filter by need; "Request proposal" (member starts it); ratings from members | v1.1 |
| **Capital** | Opt-in "raising" flag; verified Capital Partners can request an intro, which the member accepts or declines; investor office hours events | v1.1 |
| **Board & NED roles** | "Open to board seats" flag; board roles posted by companies and licensed search partners; express interest | v2 |
| **AI meeting brief** | Before any intro, meeting or event: a 1-page brief (common ground, 3 questions, recent company news) | v1.1 |
| **AI assistant ("Ask Majlis")** | "Who in the network has expanded into Saudi logistics?" Returns members (respecting privacy settings) and suggests a host intro | v2 |
| **CEO Pulse surveys** | 5-question quarterly survey in the app; members see the aggregate results first | v1.1 |
| **Recognition** | "Most helpful" badges, Founding Member, Circle Host, event speaker | v1.1 |
| **Language** | Full English and Arabic (right-to-left layout), auto-translate in chat | MVP (UI) / v1.1 (translate) |
| **Accessibility** | Dynamic type, screen reader labels, contrast AA | MVP |

---

## 9. Settings & privacy (Me → Settings)

- **Account:** email, mobile, linked sign-in methods (LinkedIn, Apple, magic link), 2-factor authentication, active devices, app lock with Face ID / fingerprint.
- **Privacy:**
  - Directory visibility: Visible / Connections only / Hidden.
  - Per-field visibility for size and money, contact details, experience and events attended.
  - Read receipts on/off; who can send connect requests (everyone / mutuals only).
  - Show me in "Who's here" at events; presence ("I'm around").
  - Visitor visibility in City Passport.
- **Intros:** frequency (3 / 2 / 1 a week), pause, preferred sectors to meet more or less of.
- **Notifications:** per category; digest day and time; quiet hours.
- **Partner content:** mute all partner-hosted promotions, while still able to attend events.
- **EA delegate** (v1.1).
- **Data:** download my data, delete my account (grace period of 30 days), consent history.
- **Help:** Member Code, FAQ, contact host, report a problem.

---

## 10. Notifications matrix

| Trigger | Push | Email | In-app | Default |
|---|---|---|---|---|
| Application status change | ✓ | ✓ | ✓ | On |
| New weekly intros | ✓ | ✓ (digest) | ✓ | On |
| Mutual intro / connect accepted | ✓ | — | ✓ | On |
| New connect request | ✓ | — | ✓ | On |
| 1-to-1 message | ✓ | — | ✓ | On |
| Group / Circle message | @mentions only | — | ✓ | Mentions |
| City Lounge | Off (digest) | Weekly digest | ✓ | Off |
| Event invite / waitlist promotion | ✓ | ✓ | ✓ | On |
| Event reminders (24h, 2h) | ✓ | — | ✓ | On |
| Someone replied to my Ask | ✓ | — | ✓ | On |
| Member I follow is travelling to my city (opt-in) | ✓ | — | ✓ | Off |
| Host message | ✓ | ✓ | ✓ | On |

Quiet hours (22:00–07:00 by default) hold back everything except event-day reminders.

---

## 11. Data model additions (on top of [05](05-technology-and-architecture.md#core-data-model-simplified))

```
connection(id, requester_id, addressee_id, note, status[pending|accepted|ignored], source[intro|request|event|card|host])
channel(id, kind[dm|group|circle|lounge|event|concierge|advisor|dealroom], city_id, circle_id, event_id, title, settings_json)
channel_member(channel_id, member_id, role[member|host|moderator], muted, last_read_at)
message(id, channel_id, sender_id, parent_id, body, attachments_json, kind[text|voice|card|meeting_proposal|system], moderation_state, created_at)
meeting_proposal(id, message_id, slots_json, place_id, chosen_slot, calendar_event_ids)
place(id, city_id, area_id, name, type, lat, lng, good_for[], perks_json, partner_id, bookable)
area(id, city_id, name)                         -- neighbourhoods (DIFC, Downtown…)
presence(member_id, area_id, until, visibility)  -- "I'm around"; no raw GPS stored
trip(id, member_id, city_id, start_date, end_date, visible_to_locals)
event_ticket(id, event_id, member_id, qr_token, checked_in_at, plus_one_name)
event_feedback(event_id, member_id, rating, met_member_ids[])
member_status(member_id, text, kind[open|travelling|busy], until)
delegate(id, member_id, ea_user_id, scopes[])   -- v1.1
saved_item(member_id, kind, ref_id)
```

---

## 12. Release plan

| Release | When | Scope |
|---|---|---|
| **MVP** (private beta → Dubai launch) | Months 3–6 | Registration request (all routes) + reviewer console · Profile + digital card · Weekly intros, connect requests, host intros · **Chat**: 1-to-1, small groups, Circles, **City Lounge**, event chat, concierge · **Events**: discover (list/calendar/map), RSVP modes, waitlist, tickets, check-in, "Who's here", post-event connect, member-hosted meetups · **Locations**: city and neighbourhood model, Places light (list + map + propose a meeting) · Asks & Offers · Directory · Notifications · Settings and privacy · English/Arabic UI |
| **v1.1** | Months 7–12 | Trusted Advisors · Capital Partner access · City Passport (travel) · "I'm around" presence · Places full (perks + meeting room booking) · auto-translate · disappearing messages · polls · EA delegate · AI meeting briefs · online sessions + recordings · knowledge library · CEO Pulse · Global Lounge · Abu Dhabi |
| **v2** | Months 12–24 | Deal rooms · board & NED marketplace · audio/video calls · end-to-end encryption · AI assistant "Ask Majlis" · seating plans · learning-to-rank matching · Riyadh and beyond |

## 13. MVP build estimate (team of 4 engineers + 1 designer)

| Workstream | Weeks (parallel tracks) |
|---|---|
| Design system (English/Arabic RTL), all MVP screens in Figma | 4 |
| Auth (LinkedIn OIDC, Apple, OTP), application flow, CV parsing | 4 |
| Admin console: verification queue, members, events, moderation | 5 |
| Profile, directory, connections, intros + matching v1 | 5 |
| Chat (all MVP channel types, attachments, voice notes, moderation hooks) | 6 |
| Events (builder, RSVP, tickets/wallet, check-in, post-event) | 5 |
| Places, map, propose-a-meeting, calendar sync | 3 |
| Notifications, digest emails, settings and privacy, data export/delete | 3 |
| QA, security test, App Store / Play review | 3 |
| **Total elapsed** | **≈ 14–16 weeks** from design start to App Store release |
