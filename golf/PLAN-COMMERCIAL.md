# Staff Golf Tournament App — Commercial Distribution Plan

**Companion to:** `PLAN.md` (the internal-use plan, which remains valid and unchanged)
**Date:** July 29, 2026
**Status:** Market assessment and revised plan for review

---

## 1. The verdict, up front

**Qualified yes — but not as a golf tournament app.**

If the plan is "build a nicer version of what already exists and sell it to golfers," the honest
answer is no. That market is crowded, and more importantly it is *commoditized by free*.
Squabbit ships 30+ formats, live leaderboards, GPS and an Apple Watch app with — in their own
words — no paywalls, no ads, and no limits on players, teams or tournaments. PlayThru is free.
GolfStatus is free for any qualifying nonprofit. You cannot win a price war against zero, and
"but ours is prettier" is not a business model. Design quality gets you noticed; it does not get
you paid when the alternative costs nothing.

**But the money in golf events is not in the scoring software.** It is in the fundraising.
Roughly 143,000 charitable golf events happen in the US each year, drawing about 12 million
participants and raising in the neighborhood of $4 billion, averaging around $26,400 per event.
GolfStatus figured this out: they give the software away and charge a **$750 flat fee, payable
only if a sponsorship sells** — and report that organizations raise about $4,000 extra by
listing it. The software is the delivery mechanism. The sponsor is the customer.

That is a real, proven, defensible model. And there is a specific slice of it where you have an
advantage almost nobody else has.

**The wedge: faith-based and small-nonprofit outings.** Church golf outings are everywhere —
men's ministry events, building-fund fundraisers, staff retreats, youth mission trip
fundraisers. They are almost always run by a volunteer or a staff member with no events
background, on a budget that cannot absorb $1,300/year for Golf Genius. The incumbents either
ignore this segment or serve it with generic tooling. You are a church IT director. You will be
running these events yourself, you know the vocabulary, and church networks are unusually
high-trust and referral-driven — one good outing at a regional gathering is worth more than a
month of ads.

**What this realistically is:** a well-built side business that could plausibly reach
$3,000–8,000/month within about two years of consistent effort. It is not a venture-scale
company, and I would not plan as though it were. It is also low-risk: the app gets built
anyway for your own use, so the incremental cost of testing the market is a $99 developer
account and the time to polish.

**The one hard technical consequence:** the PWA plan in `PLAN.md` will not survive App Store
review as-is. See §7.

---

## 2. Competitive landscape — the honest picture

### The free tier is brutally good

| Product | Price | What it does | Weakness |
|---|---|---|---|
| **Squabbit** | Free, no limits | 30+ formats, live leaderboards, GPS, handicaps, Apple Watch app | Dated UI; built for golfers, not for fundraising organizers |
| **PlayThru** | Free | Live leaderboards, scramble-focused, simple | Thin feature set; little organizer tooling |
| **GolfStatus** | Free for nonprofits | Full event site, registration, payments, live scoring, live support | Nonprofits only, must qualify; sponsor-fee model; heavier setup |
| **DoJiggy** | Free tier | Tournament websites, registration | Fundraising-first, scoring is secondary |

### The paid tier is expensive and aimed elsewhere

| Product | Price | Aimed at |
|---|---|---|
| **Golf Genius** | ~$1,300/year | PGA sections, USGA clubs, serious operations |
| **VPAR** | Enterprise/event pricing | Corporate events, societies, 50+ formats, on-site hardware |
| **Enterprise suites** | $500–$2,000/month | Full course management, tournaments as one module |

### What this tells us

There is a genuine gap in the middle. Below Golf Genius's $1,300 there is essentially nothing
that combines *good* scoring with *good* fundraising tooling and a *modern* interface. The free
products are good at scoring and weak at organizing. The expensive products are good at both
and priced for institutions. The nonprofit-free products require qualification and are built
around their own sponsor economics.

**But be clear-eyed about what that gap means.** It exists partly because it is hard to make
money there. The gap is real; it is not obviously lucrative. That is why the plan below
monetizes the *event*, not the *software*.

---

## 3. Where the money actually is

Three revenue mechanisms are viable here, ranked by how much I believe in them:

### Tier 1 — Sponsor-linked revenue (the real business)

Follow the GolfStatus playbook, undercut it, and serve the segment they don't. Every golf
outing sells sponsorships — hole signs, cart sponsors, beverage carts. A digital leaderboard
that 40–150 people stare at all day for four hours is genuinely valuable ad inventory, and it
is inventory the organizer currently cannot sell because they have no place to put it.

**The offer:** "Add a Presenting Technology Sponsor to your event. Their logo appears on the
leaderboard every player checks a dozen times, on the results email, and on the shareable
final scorecard. You keep everything above our fee." Price it at **$299–$399, charged only if
the sponsorship sells.** GolfStatus charges $750; at half that, aimed at events raising
$10–30K rather than $100K+, this is an easy yes for an organizer who was going to sell the
sponsorship anyway.

This aligns incentives perfectly: you make money only when the organizer makes more money.
That is a very easy thing to sell into a church or small nonprofit, and it sidesteps the "we
don't have a software budget" objection entirely, because it is not a software purchase.

### Tier 2 — Organizer subscription (steady but modest)

For people running recurring events — leagues, monthly men's golf, corporate series — a flat
subscription. **$79/year or $12/month**, unlimited events, unlimited players. Priced deliberately
far under Golf Genius and framed as "for the person running it, not for the players."

Players are always free. Always. Charging players is the single fastest way to kill adoption,
because the organizer then has to justify it to 60 people in an email.

### Tier 3 — Player premium (low conviction, build last)

A small consumer subscription — $19.99/year — for season-long stats across events, personal
handicap tracking, history, custom cards. I include it for completeness but I would not count
on it. 18Birdies charges $60–90/year and has a decade of brand and millions of users; competing
for that same wallet is not our fight.

### What I would explicitly *not* do

- **Ads.** Destroys the premium feel entirely and yields pennies at this scale.
- **Charging players.** Kills the network effect that makes the app work at all.
- **Taking a cut of registration fees.** Puts you in payments compliance, PCI scope, and
  nonprofit money handling. Enormous complexity for modest margin. Let organizers keep using
  whatever they already use to collect money.
- **Free-for-everyone with no plan.** You will build a support burden with no revenue and quit
  in eight months.

---

## 4. Positioning

**"Tournament software that looks like Apple made it — free for players, and it helps you raise
more."**

Three things to be genuinely, visibly better at than the free competition:

1. **Design.** This is the honest, defensible edge and it is the thing `PLAN.md` already
   specifies. The free tools work but look like 2014. In a market where every competitor's
   screenshots look the same, an app that looks like it shipped from Cupertino wins the
   App Store listing before anyone reads a word. Design is a weak moat long-term but an
   excellent *wedge*.

2. **Setup time.** The organizer's real pain is not scoring, it is the two hours of
   configuration the night before. Target: **a full 72-player shotgun scramble configured in
   under five minutes.** Course library with pre-loaded ratings and stroke indexes, paste a
   list of names from a spreadsheet, one tap to generate balanced pairings. That is a
   demonstrable, screenshot-able, benchmark-able claim.

3. **Nobody has to install anything.** This is the sleeper feature and it deserves to be the
   headline. The organizer installs the app. **Players just open a link.** No app store, no
   account, no 60-person email chain about downloading something. Every competitor requires
   every participant to install their app, and every organizer has been burned by the four
   people who never did. Making the web app a first-class citizen rather than a fallback is a
   real structural advantage — and it is the natural consequence of `PLAN.md` already being
   web-first.

**Vertical messaging for the wedge:** a dedicated landing page and App Store keyword set for
church and ministry outings. Same product, language that sounds like it was written by someone
who has actually run one.

---

## 5. Go-to-market

**Phase A — Prove it works (months 1–4).** Run your own staff outings on it. Then your church's
next fundraiser. Then offer it free to five other churches in your network in exchange for
blunt feedback and permission to quote them. Fix everything that breaks. This is not marketing,
it is product development, and it is the most valuable thing on this list.

**Phase B — Quiet launch (months 5–7).** Ship to both stores. Free tier live. Sponsor feature
live. Ask the churches from Phase A to try selling a technology sponsorship — that is your
first revenue and your first case study with a real dollar figure attached.

**Phase C — Deliberate growth (months 8–18).**
- Content aimed squarely at the search intent: "how to run a church golf tournament,"
  "golf scramble pairings," "how to handicap a golf outing with no handicaps." Organizers
  research this in the weeks before their event, and there is a free handicap calculator and
  pairing generator hiding inside your app that makes excellent link bait.
- Church network channels: denominational gatherings, church IT and admin communities,
  ministry conference vendor tables.
- App Store Optimization on the long tail — "golf scramble app," "golf outing scoring,"
  "charity golf tournament" — rather than the hopeless head term "golf app."
- A referral loop that already exists: every player who uses the leaderboard sees a small,
  tasteful "run your own outing" affordance on the results screen. Half of them organize
  something. This is your cheapest acquisition channel and it costs one screen.

**Seasonality to plan around:** this business is violently seasonal. US outings cluster
April–October and peak in May, June and September. Ship before March. Do maintenance and
build features in winter. Budget for the fact that January revenue will look like failure and
is not.

---

## 6. Realistic numbers

Stated as ranges with the reasoning attached, because precise projections here would be
fiction.

### Costs

| Item | Cost |
|---|---|
| Apple Developer Program | $99/year |
| Google Play Developer | $25 one-time |
| Domain | ~$15/year |
| Supabase (free tier → Pro when needed) | $0 → $25/month |
| Email delivery (Resend/Postmark) | $0 → ~$20/month |
| Landing page hosting | $0 (Cloudflare Pages) |
| **Year one total** | **~$400–900** |

Trivial. The real cost is your time, and much of it is spent anyway building the internal tool.

### Revenue scenarios, year two

| Scenario | Sponsor events | Subscriptions | Annual | Monthly |
|---|---|---|---|---|
| **Conservative** | 60 × $349 | 150 × $79 | ~$32,800 | ~$2,700 |
| **Base** | 180 × $349 | 400 × $79 | ~$94,400 | ~$7,900 |
| **Optimistic** | 500 × $349 | 1,200 × $79 | ~$269,300 | ~$22,400 |

For context on plausibility: the base case is roughly **0.13% of the ~143,000 annual US
charity golf events**. That is a small number and it is still a real income. The optimistic
case is about 0.35% and would require this to become a genuine second job.

I would plan for conservative, hope for base, and treat optimistic as a pleasant surprise
rather than a forecast.

### Store commissions

Apple and Google each take 15% under their small-business programs (under $1M/year). Two
important structural details work in our favor:

- **Event registration and sponsorship fees are real-world services**, which under Apple's
  Guideline 3.1.3(e) *must not* use in-app purchase — they use ordinary card payment. So the
  Tier 1 sponsor revenue, the main business, **is not subject to store commission at all.**
- **Organizer subscriptions are digital** and would normally require IAP at 15%. Selling them
  on the web instead is the standard route, and following the 2025 Epic injunction US apps may
  now link out to external purchase pages — though Apple won the right on appeal in December
  2025 to charge some commission on those links, and the Supreme Court has yet to settle the
  rate. **Treat this as unsettled and verify before building against it.** Simplest safe
  approach: offer subscriptions through IAP inside the app and eat the 15%, while also selling
  on the web to anyone who arrives there first.

---

## 7. What changes technically

This is the section with real consequences for the build.

### The PWA cannot ship to the App Store

`PLAN.md` specifies a Progressive Web App. That is exactly right for internal use and it is a
**rejection under Apple Guideline 4.2 (Minimum Functionality)**. Apple's language is that an
app "should include features, content, and UI that elevate it beyond a repackaged website,"
and reviewers reject wrapped web views with "your app is not sufficiently different from a
mobile web browsing experience." A PWABuilder or Capacitor wrapper with no native substance is
the textbook rejection case.

### The fix: React Native via Expo, sharing the scoring engine

| Option | Verdict |
|---|---|
| PWA wrapped in Capacitor | ✗ High rejection risk under 4.2 unless substantial native features are added |
| **React Native (Expo)** | **✓ Recommended** — genuinely native UI, one codebase for iOS and Android, shares the TypeScript scoring engine with the web app, over-the-air updates, passes 4.2 without argument |
| Native SwiftUI + Kotlin | ✓ Best possible quality, but roughly double the work and two codebases to maintain forever |

**Expo is the right call**, and the reason is that `PLAN.md` already made the correct
architectural decision: the scoring engine is a pure TypeScript module with no UI or database
dependencies. That module is now shared, unchanged, across three surfaces:

```
                  ┌──────────────────────────┐
                  │   @fairway/scoring       │
                  │   pure TypeScript        │
                  │   formats · handicaps    │
                  │   brackets · pairings    │
                  └────────────┬─────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
   ┌──────▼──────┐     ┌───────▼───────┐    ┌──────▼──────┐
   │ Expo app    │     │ Web app       │    │ Supabase    │
   │ iOS/Android │     │ join by code  │    │ edge funcs  │
   │ organizers  │     │ no install    │    │ validation  │
   │ + players   │     │ players       │    │             │
   └─────────────┘     └───────────────┘    └─────────────┘
```

**The web app does not go away — it gets more important.** It becomes the no-install path for
players, which §4 identifies as a headline differentiator. The native app is for organizers and
for players who want the better experience. This is a strength, not a compromise.

### Native features that both satisfy Apple and are genuinely useful

Guideline 4.2 wants real app-like capability. Conveniently, the things that clear review are
the same things that make the product better:

- **Push notifications** — "You're now in 2nd place," "Group 4 has finished," "Your tee time is
  in 30 minutes." Impossible to do well on iOS web; a real reason to install.
- **Apple Watch companion** — enter scores from your wrist without pulling a phone out of your
  pocket mid-round. Squabbit has this and players genuinely love it. Strong 4.2 evidence.
- **Live Activity / Dynamic Island** — your current score and position on the lock screen for
  the whole round. This is a genuinely delightful, very Apple feature that no competitor has,
  and it would carry the App Store screenshots on its own.
- **Native offline storage** — SQLite instead of IndexedDB. More robust and, since golf courses
  have no signal, more correct.
- **Camera** — photograph your closest-to-the-pin claim.
- **Haptics** — a distinct tap for birdie versus bogey.
- **Share sheet** — post the final leaderboard card.
- **Siri Shortcuts / widgets** — "Hey Siri, what's my score?"

Each of these is defensible under 4.2 and each is a real reason to install rather than use the
website.

### Also required for public release

- Real authentication for organizer accounts — Sign in with Apple (mandatory if any other
  social login is offered), Google, and email magic link. The anonymous code-join stays exactly
  as designed for players; that is the product.
- Multi-tenancy hardening. Row Level Security policies now protect strangers' data, not
  colleagues'. This deserves an actual security review, not a glance.
- Rate limiting and abuse prevention on code join — someone will brute-force 6-character codes.
  Lengthen codes for public tournaments and rate-limit attempts per IP.
- Privacy policy, terms of service, App Privacy nutrition labels, support URL, account deletion
  in-app (Apple requires it if accounts can be created).
- Error monitoring (Sentry) and product analytics (PostHog) — you cannot fix what you cannot
  see, and with real users you no longer get to just ask them.
- A course database. This is the quiet, unglamorous, genuinely large task. Ratings, slopes,
  pars and stroke indexes for thousands of courses. Start with crowd-sourced entry — the first
  organizer at a course enters it, everyone after inherits it — and seed the top few hundred US
  courses by hand. Over time this becomes the most valuable asset you own and the hardest thing
  for a new competitor to replicate.

---

## 8. Revised roadmap

Phases 1–7 from `PLAN.md` stay exactly as written and remain the foundation. What follows is
what gets added, and it only begins after the internal app has survived real outings.

| Phase | Weeks | Content |
|---|---|---|
| **1–7** | 1–8 | *Everything in `PLAN.md`.* Web app, scoring engine, offline, brackets, polish. |
| **8. Prove it** | 9–14 | Run real outings. Your staff, your church, five friendly churches. Fix what breaks. **Decision gate — see §10.** |
| **9. Native shell** | 15–19 | Extract the scoring engine into a shared package. Expo app for iOS and Android. Port screens to native. Push notifications, haptics, native offline store. |
| **10. Store-ready** | 20–23 | Organizer auth, multi-tenancy hardening, security review, privacy policy, terms, App Privacy labels, account deletion, analytics, monitoring. |
| **11. Money** | 24–27 | Sponsor branding system (leaderboard, results email, shareable card). Subscription billing via Stripe and IAP. Organizer billing dashboard. |
| **12. Delight** | 28–31 | Apple Watch app. Live Activities. Widgets. Share cards. The features that carry the store listing. |
| **13. Launch** | 32–35 | Landing page, App Store assets and screenshots, ASO, launch content, five case studies from Phase 8. |
| **14. Course data** | ongoing | Seed the top US courses. Build crowd-sourced entry and moderation. Never really finishes. |

**Roughly nine months of part-time work to a real commercial launch.** Note that the internal
app you actually wanted is done and in use at week 8, and everything after that is optional and
independently abandonable. That is the best possible risk structure for a project like this:
you get the thing you needed regardless of whether the business works.

---

## 9. Risks, stated plainly

| Risk | Severity | Honest assessment |
|---|---|---|
| **Free competitors** | **High** | The central risk. Squabbit and PlayThru are free with no limits. Mitigation is to not compete on scoring — compete on setup speed, design, and fundraising outcomes, and monetize sponsors rather than software. If organizers won't pay for the sponsor feature, there is no business. |
| **Seasonality** | High | Revenue concentrates April–October. Cash flow will look alarming in January. Plan for it; don't panic. |
| **Course data cost** | Medium | Thousands of courses, each needing accurate ratings and stroke indexes. Crowd-sourcing works but is slow and needs moderation. |
| **Support burden** | Medium | Live events mean a Saturday morning "the leaderboard is frozen and 80 people are waiting" call. That is a real personal cost, on weekends, during your own church's busy season. Take this seriously before committing. |
| **Incumbent response** | Medium | If it works, GolfStatus or Squabbit can copy the design in a quarter. The moats are course data, organizer relationships, and switching cost — none of which exist on day one. |
| **App Store rejection** | Low | Manageable if built as Expo with real native features. Would be near-certain with a wrapped PWA. |
| **Your time** | **High** | You have a full-time job in church IT. This is nights and weekends for nine months before a dollar arrives. This is the risk most likely to actually end the project, and it deserves more weight than any of the technical ones. |
| **Liability and payments** | Low | Avoided by design — we never touch registration money. Keep it that way. |

---

## 10. Decision gates

Three explicit checkpoints. Each has a stated condition, and each has a legitimate stop that
still leaves you better off than when you started.

**Gate 1 — after week 8 (internal app done)**
*Question:* Do our own outings actually run better on this?
*Continue if:* players used it unprompted and the organizer's setup time genuinely dropped.
*If not:* stop. You have a great internal tool. That was the original request and it is a
complete, successful outcome.

**Gate 2 — after week 14 (five friendly churches)**
*Question:* Did organizers who owe you nothing choose to use it again?
*Continue if:* at least three of five would run their next event on it, and at least two say
unprompted that they would pay something.
*If not:* stop, and keep using it internally. You will have lost six weeks and learned a real
answer cheaply. This is the most important gate and the one to be most honest at.

**Gate 3 — six months post-launch**
*Question:* Is there revenue and organic growth?
*Continue if:* 25+ paying events, some arriving without you personally recruiting them.
*If not:* keep it alive as a free tool with near-zero maintenance, or open-source it. Do not
spend another year pushing a rock uphill.

---

## 11. Recommendation

**Build the internal app first, exactly as `PLAN.md` describes. Decide about the business at
Gate 2, with real data instead of a spreadsheet.**

The asymmetry here is unusually good. You are building this anyway. The additional cost of
finding out whether it is a business is $124 in developer fees and a few months of evenings,
and the downside case still leaves you with the tournament app you wanted, running your own
outings.

What makes me think it might genuinely work is not the market size — the market is crowded and
the free tier is strong. It is three specific things:

1. **You are the customer.** You will run these events yourself, repeatedly, and feel every
   rough edge. That is worth more than any amount of user research, and it is the thing most
   failed products in this category lacked.
2. **The sponsor model is proven and undefended at the low end.** GolfStatus validated that
   organizers will pay when payment is contingent on them making more money. Nobody is serving
   the $10–30K event with that model.
3. **You have distribution that money cannot buy.** Church networks are high-trust,
   referral-heavy, and nearly impossible for an outside vendor to penetrate. You are already
   inside one.

What would make me say no: if at Gate 2 organizers like the app but shrug at the sponsor
feature. That is the whole thesis. If the sponsorship doesn't sell, there is no business here —
only a very nice free tool, which is still a perfectly good thing to have made.

**One practical note before anything else:** "Fairway" is almost certainly taken in this
category. Run a USPTO and App Store name search early, before any design work bakes a name into
icons and screenshots.

---

## Sources

- [Tournament Management Pricing — Golf Genius](https://golfgenius.com/products/tm/pricing)
- [Squabbit — Free Golf Tournament & League Software](https://squabbitgolf.com/)
- [Squabbit — App Store listing](https://apps.apple.com/us/app/squabbit-golf-tournament-app/id1556538444)
- [PlayThru — Golf Scorecard App and Live Leaderboards](https://www.golfplaythru.com/)
- [GolfStatus — Golf Tournament Management Software](https://golfstatus.com/)
- [GolfStatus — Charity Golf Tournament Technology Sponsorship](https://golfstatus.com/technology-sponsor)
- [The 12 Best Golf Tournament Software Solutions for 2026 — Live Tourney](https://www.livetourney.com/blog/best-golf-tournament-software)
- [Golf Event Management Software: A Buying Guide — Golf Course Technology Reviews](https://www.golfcoursetechnologyreviews.org/buying-guide/golf-event-management-software-buying-guide-for-2025)
- [Golf Generates $3.9 Billion Annually for Charity — Ohio Golf Journal](https://ohiogolfjournal.com/golf-generates-3-9-billion-annually-for-charity/)
- [Charity Golf Tournament Guide — Momentive Software](https://momentivesoftware.com/blog/charity-golf-tournament/)
- [18Birdies Review 2026: Is Premium Worth $60–$90/yr? — Scoring Zone](https://www.scoringzone.net/blog/18birdies-review.html)
- [Golf App Subscription Cost 2026 — Scoring Zone](https://www.scoringzone.net/blog/golf-app-subscription-cost.html)
- [App Store Review Guidelines — Apple Developer](https://developer.apple.com/app-store/review/guidelines/)
- [Can You Publish a PWA to the App Store and Google Play? — MobiLoud](https://www.mobiloud.com/blog/publishing-pwa-app-store)
- [App Store Review Guidelines: Will Your Webview App Be Rejected? — MobiLoud](https://www.mobiloud.com/blog/app-store-review-guidelines-webview-wrapper)
- [App Store Guideline 4.2 Minimum Functionality — TechNet Experts](https://www.technetexperts.com/guideline-4-2-minimum-functionality/)
- [Apple Wins Ability to Charge Fees on External Payment Links — MacRumors](https://www.macrumors.com/2025/12/11/apple-app-store-fees-external-payment-links/)
- [Apple must allow External Payment Links: What the Epic ruling means — RevenueCat](https://www.revenuecat.com/blog/growth/apple-anti-steering-ruling-monetization-strategy)
- [Epic vs. Apple heading to the Supreme Court, again — AppleInsider](https://appleinsider.com/articles/26/04/06/epic-vs-apple-lawsuit-over-app-store-fees-is-moving-to-the-supreme-court-again)
- [External Payment Links iOS Apps: What Devs Must Know — Twinr](https://twinr.dev/blogs/external-payment-links-ios-app/)
