# 06 — Trust, Safety & Legal

> This is a planning checklist, not legal advice. Engage UAE counsel (ideally a firm that could also become a Founding Partner) before launch.

## Company setup (UAE)
- **Entity:** a free-zone company suited to tech platforms and events, for example **DIFC Innovation Hub** (common-law courts, DIFC Data Protection Law, strong credibility with investors and banks), **Dubai Silicon Oasis / in5**, or **DMCC**. Choose based on investor preference and banking.
- **Licence activities:** online platform / social networking services, event management, business consultancy (for partner services). Check that the activity list covers taking sponsorship and listing fees.
- **Events:** sponsored events at hotels and venues may need permits from Dubai's **Department of Economy and Tourism (DET)**. The venue usually helps; budget for it per event.
- **VAT:** register for UAE VAT (5%) once taxable supplies pass the AED 375k mandatory threshold. That will happen in year 1 because of sponsorship revenue.
- **Corporate tax:** UAE corporate tax of 9% applies above AED 375k profit. Free-zone "qualifying income" rules may change this, so get tax advice on the structure.

## Data protection
| Regime | When it applies | Key obligations |
|---|---|---|
| **UAE PDPL** (Federal Decree-Law No. 45 of 2021) | Processing in the onshore UAE | Lawful basis/consent, data subject rights (access, correction, deletion, portability), breach notification, cross-border transfer only to adequate jurisdictions or with safeguards |
| **DIFC Data Protection Law No. 5 of 2020** | If the entity is in DIFC | GDPR-like: DPO for high-risk processing, records of processing, DPIAs, notification to the Commissioner, transfer rules |
| **ADGM Data Protection Regulations 2021** | Abu Dhabi expansion via ADGM | GDPR-like |
| **KSA PDPL** | Riyadh launch | Registration with SDAIA's platform, strict cross-border transfer rules, possibly data localisation |
| **UK GDPR / EU GDPR** | London launch, EU residents | Standard GDPR plus international transfer mechanisms |

**Design choices that make compliance easier**
- **Data minimisation:** delete CVs and KYC documents after the decision, with a maximum of 30 days by default.
- **Explicit, granular consent** for anything beyond core matching: partner programmes, Capital Partner discoverability, board placement, CEO Pulse participation.
- **UAE-region hosting** for primary data, with documented transfer safeguards for any sub-processors such as the AI API and email.
- **AI processing:** use an LLM provider under terms that don't train on customer data. Document it in the privacy notice and keep a record of AI-assisted decisions. A human makes every verification decision.
- **Self-serve privacy centre:** export my data, delete my account, see who viewed my profile (optional), manage consents.

## Regulated activities to avoid (or license through partners)
| Activity | Risk | How we handle it |
|---|---|---|
| Matching founders with investors (Capital Partners) | May count as **arranging deals in investments** or a financial promotion under DFSA (DIFC), FSRA (ADGM), SCA (onshore UAE) or VARA (for virtual assets) | Intro-only model: flat seat fees, **no success fees on capital raised**, no advice and no handling of money. Get a legal opinion; partner with a licensed placement firm if we move to success fees |
| Board and executive placement fees | Recruitment and employment agency activity can require licensing (for example MOHRE private employment agency rules onshore) | Run placements **through licensed executive search partners** on revenue share; the platform only provides opt-in discovery |
| Insight reports | Must not identify individuals or companies | Aggregate only, minimum group size of 20, legal review of every report |
| Member-to-member deals in deal rooms | Members exchanging inside information on listed companies | Terms and an in-app reminder about insider-trading laws; no investment recommendations from the platform |

## Platform rules & moderation
- **Member Code** ([02](02-features-and-ux.md#member-code-shown-at-onboarding-and-in-settings)) accepted at onboarding.
- **Reporting:** a "Report" control on every profile, message, Ask and event. Reports reach a human within 24 hours.
- **Enforcement ladder:** a nudge, then a warning, then a temporary pause, then removal. Removals are reviewed by two staff.
- **AI-assisted triage** flags sales language, spam patterns and policy breaches in Asks and Circle posts, but **a human decides** every action.
- **Anti-impersonation:** each LinkedIn `sub`, phone number and work email can belong to only one account. Profile photos are checked against the LinkedIn photo or the liveness selfie.
- **Partner conduct:** contractual ban on contacting members outside the platform's opt-in flows. One breach can end the contract.
- **Safety for members:** block and mute, hide from directory, quiet hours, and no visible location beyond city.

## Key legal documents to prepare
1. Terms of Membership (including Member Code and Chatham House Rule).
2. Privacy Notice (PDPL / DIFC / GDPR versions) and Cookie Policy.
3. Partner Agreement templates: City Partner, Event Sponsor, Trusted Advisor, Capital Partner, Search Partner. All include the data and no-solicitation clauses.
4. Data Processing Agreements with each sub-processor.
5. Event terms and photo consent.
6. Internal: data retention schedule, breach response runbook, records of processing, DPIA for verification and matching.
