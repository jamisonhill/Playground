# Staff Golf Tournament App — Project Plan

**Working name:** Fairway
**Date:** July 29, 2026
**Status:** Plan for review

---

## 1. Viability: Yes, comfortably

This is a well-scoped project. Nothing here requires exotic technology. For a dozen-plus
players across 18 holes, the data volume is tiny — a full 4-round-per-group tournament with
24 players produces about 432 score rows. That is nothing. The interesting engineering is not
scale, it is these three things:

1. **Golf courses have terrible cell signal.** This is the single biggest technical risk and
   the reason most homegrown scoring apps fail. The app must work with no connection and sync
   when signal returns. This is designed in from day one, not bolted on.
2. **Scoring rules are fussy.** Scramble, best ball, Stableford and match play each score
   differently, and handicap allowances differ per format. This deserves a real, tested
   scoring engine rather than scattered `if` statements.
3. **Zero-friction access.** Staff will not create accounts in a parking lot. Access is a
   tournament code plus picking your name from a list. That is it.

Everything else — leaderboards, brackets, pairings, light/dark — is standard web app work.

**Realistic estimate:** a genuinely usable version for one outing in roughly 3–4 weeks of
part-time evenings. The full plan below lands around 7–8 weeks part-time. It can be used
after Phase 3.

**Running cost:** $0. Supabase free tier and Cloudflare Pages free tier cover a group this
size with enormous headroom.

---

## 2. What we are building

A mobile-first Progressive Web App. No app store, no install required, but it can be added to
the home screen and will then look and behave like a native iOS app — full screen, own icon,
no browser chrome.

The flow, end to end:

- **You (admin)** create a tournament, pick the course, add players, choose a format, and
  generate pairings. The app gives you a 6-character code.
- **Everyone else** opens a link, types the code, taps their name, and they are in. No
  password, no signup.
- **On the course**, one person per group enters scores hole by hole. Everyone else watches
  the leaderboard update live.
- **Afterward**, final standings, skins, closest-to-pin, and a shareable results card.

---

## 3. Feature research — what actually matters

I looked at what Golf Genius, 18Birdies, Golf Pad, Squabbit and VPAR ship, and at what
tournament organizers complain about. Sorting the findings into what we build and what we
deliberately skip:

### Must have (the app is not useful without these)

| Feature | Why |
|---|---|
| Tournament code join, no account | The whole point. Friction here kills adoption. |
| Live leaderboard, auto-updating | The reason people open the app between shots. |
| Group scorecard entry | One scorer per foursome enters for everyone — this is how golf actually works. |
| "Thru 12" progress indicator | Without it a leaderboard is meaningless — a player at −2 thru 4 is not beating −1 thru 17. |
| Offline entry with auto-sync | Non-negotiable. See §9. |
| My Card view | Your own round, front nine / back nine / total, at a glance. |
| Multiple formats | Staff outings rotate between scramble and individual play. |
| Light and dark mode | Requested. Also genuinely useful — bright sun and a dim clubhouse. |

### Should have (what makes it feel finished)

- **Pairings and tee sheet** — who is in your group, what hole you start on, what time.
- **Shotgun start support** — most outings of this size use one. Every group starts on a
  different hole simultaneously. This affects leaderboard math and is often gotten wrong.
- **Handicaps** — including a fallback for staff who have never had a handicap index (§7).
- **Side contests** — skins, closest to the pin, longest drive. Cheap to build, and they are
  what people actually talk about afterward.
- **Match play brackets** — requested, and the natural format for a season-long staff ladder.
- **Score attestation** — a second player taps to confirm the group's card. Prevents the
  "that was a 6 not a 5" argument.
- **Live position change alerts** — a quiet toast when someone passes you. Small touch,
  disproportionate engagement.

### Nice to have (later, if wanted)

- Photo attached to a closest-to-pin claim
- Season-long standings across multiple outings
- Stat tracking (putts, fairways, GIR)
- Team logos / player avatars
- Export final results to PDF or a shareable image card

### Deliberately skipping

- **GPS rangefinder / course maps.** Enormous effort, needs licensed course data, and every
  player already has an app that does it. Not our job.
- **Payments and betting settlement.** Church staff outing. Skins can be tracked for bragging
  rights without moving money through software.
- **Native iOS/Android apps.** A well-built PWA is indistinguishable here and avoids app store
  review entirely.
- **Full USGA/WHS handicap compliance.** We will implement the standard math correctly, but we
  are not chasing certification for a staff outing.

---

## 4. Access model — roles and the tournament code

Three roles, kept intentionally simple:

**Admin (one person per tournament)**
Creates the tournament and holds an *admin code* — a separate, longer secret shown only once
at creation and recoverable from their own device. Can configure everything, edit any score,
regenerate pairings, lock the tournament when it is over.

**Scorer (one per group, assigned by admin)**
Everything a player can do, plus can enter and edit scores for every member of their group.
The admin can promote any player to scorer on the fly — useful when the assigned scorer's
phone dies on hole 7.

**Player (everyone else)**
Joins with the 6-character *tournament code*. Can see their own card, the full leaderboard,
pairings, the bracket, and side contests. Can propose a score correction on their own card,
which surfaces to their group's scorer for approval. Cannot edit anyone else's score.

**How identity works without accounts:** on join, the device is issued a signed token bound to
`(tournament, player)` and stored in local storage. That token is the credential. Clearing the
browser means rejoining with the code — acceptable for a one-day event. The admin can see
which names are claimed and unclaim one if someone taps the wrong name.

**Code format:** 6 characters from an unambiguous alphabet (no `0/O`, no `1/I/L`) — e.g.
`GOLF7K`. Short enough to read aloud across a parking lot. Codes expire 7 days after the
tournament's end date, and a joinable QR code is generated for the same link.

---

## 5. Formats and the scoring engine

This is the heart of the app and gets built as a **pure, standalone TypeScript module** with
no UI or database dependencies — just `(scores, config) → standings`. That makes it fully
unit-testable, which matters because scoring bugs are the thing that will destroy trust in the
app fastest.

### Formats supported at launch

| Format | Players | How it scores |
|---|---|---|
| **Stroke play** | Individual | Lowest total strokes. Gross and net leaderboards. |
| **Stableford** | Individual | Points vs. par: double bogey or worse 0, bogey 1, par 2, birdie 3, eagle 4, albatross 5. Highest wins. Great for outings — a blow-up hole caps your damage and nobody has to pick up in shame. |
| **Modified Stableford** | Individual | Same idea, aggressive point values (e.g. eagle +5, birdie +2, par 0, bogey −1, double −3). Rewards risk. |
| **Scramble** | Teams of 2–4 | Everyone tees off, team picks the best shot, all play from there, repeat. One team score per hole. The default for mixed-ability staff outings — nobody is exposed. |
| **Shamble** | Teams of 2–4 | Best drive is shared, then everyone plays their own ball in. Combines the safety of a scramble with individual scoring. |
| **Best ball / Four-ball** | Teams of 2–4 | Everyone plays their own ball; the team takes the lowest score on each hole. Configurable to count the best 1 or best 2 scores. |
| **Match play** | 1v1 or 2v2 | Hole-by-hole. Win a hole, go 1 up. Displayed as "3 & 2". Feeds the bracket. |

### Design of the engine

Each format is a strategy object exposing the same interface:

```
computeHoleResult(holeScores, config) → per-hole points or strokes
computeStandings(allRounds, config)   → ordered leaderboard rows
formatDisplay(row)                    → "−3", "34 pts", "3 & 2"
```

Adding a format later (Chapman, Wolf, Nassau) means adding one strategy and its tests — no
changes anywhere else in the app.

### Tie-breaking

Configurable per tournament, defaulting to the standard USGA countback: back nine, then last
six, last three, then 18th hole. Falls back to a declared tie if still level, with an option
for the admin to mark a playoff result manually.

---

## 6. Pairings and brackets

### Pairings

The admin picks a pairing method and the app generates a tee sheet, which can then be
drag-and-drop adjusted by hand — the generator is a starting point, never a straitjacket.

- **Random** — shuffle into groups of the chosen size.
- **Balanced by handicap** — snake draft, so each group has a strong and a weak player. This
  is what you want for a scramble.
- **Manual** — build the groups yourself.
- **Blind draw** — pairings hidden until the admin publishes them.

Supports **tee times** (groups go off #1 at intervals) and **shotgun start** (every group
starts on a different hole at the same time). Shotgun is the one that trips up naive
implementations: a group starting on hole 14 plays 14→18→1→13, so "holes completed" is not
"highest hole number entered." The engine tracks a starting hole per group and computes
progress relative to it.

### Brackets

For match play events. The admin picks a size — 4, 8, 16 or 32 — and the app builds a single
elimination bracket with standard seeding (1 vs 16, 8 vs 9, and so on), giving byes to the top
seeds when the field is not a power of two. Seeding is by handicap, by a prior stroke play
qualifying round, or manual.

The bracket is a **live view**: as a match's holes are entered, the bracket cell shows the
current state ("Smith 2 up thru 11"), and when the match is mathematically closed it
auto-advances the winner to the next round. Also supports **consolation brackets** so
first-round losers still play something, which matters for a staff outing where everyone drove
out and wants a full day.

Rendered as a horizontally-scrolling column layout on phones rather than a squeezed tree —
readable at arm's length, which is the actual use case.

---

## 7. Handicaps — including the players who don't have one

Half of a church staff will not have a handicap index. This needs to be handled gracefully or
the net leaderboard becomes a joke.

**Three ways a player gets a handicap:**

1. **Enter a Handicap Index** — for the players who have one. The app computes Course Handicap
   from the tee played:
   `Course Handicap = Index × (Slope ÷ 113) + (Course Rating − Par)`

2. **Estimate from typical score** — "I usually shoot around 95." Rough, but far better than
   nothing, and it takes one tap.

3. **One-day handicap, computed after the round** — for outings where nobody has an index.
   Two proven systems, admin's choice:
   - **Callaway System** — takes a player's worst holes, adjusted by a published chart, and
     subtracts them. Transparent; a player can verify their own.
   - **Peoria System** — the admin secretly designates six holes before play; those scores run
     through a formula to produce each player's allowance. Because the holes stay hidden until
     the end, it is the hardest to game. Use this one if sandbagging is a concern.

**Handicap allowances by format** are applied automatically per the World Handicap System
recommendations, and every value is admin-overridable:

| Format | Allowance |
|---|---|
| Individual stroke play | 95% of Course Handicap |
| Individual Stableford | 95% |
| Four-ball stroke play | 85% |
| Four-ball match play | 90% of the difference from the low player |
| Singles match play | 100% of the difference |
| Scramble, 4 players | 25% / 20% / 15% / 10% (A/B/C/D), summed |
| Scramble, 2 players | 35% / 15% |
| Shamble (best ball in) | 85% |

Strokes are allocated hole by hole using the scorecard's stroke index, so the net card shows a
little dot on the holes where you get a stroke — the way a real scorecard does.

---

## 8. Screens

Five tabs in a bottom tab bar, iOS-style, with a large-title header that collapses on scroll.

**Join** (pre-tournament) — big code field, monospaced, auto-uppercasing. Then a list of player
names to claim. Two taps to be in.

**Play** — the primary screen. One hole at a time, full width. Large `−` and `+` stepper around
a big score number, sized for a thumb in the sun. Quick-tap chips for par / birdie / bogey.
Swipe left and right between holes. Running total and score-to-par pinned at the top. In a
scramble, shows whose drive was used. In match play, shows the match state prominently. A
persistent sync indicator sits in the corner — green check when synced, amber cloud with a
count when queued offline.

**Leaderboard** — segmented control across the top: Gross · Net · Team · Skins. Rows show
position, name, score to par, and Thru. Your own row is highlighted and sticks to the bottom
of the screen when scrolled out of view, so you can always see where you stand. Tapping a row
expands their full hole-by-hole card inline. Position changes animate — rows slide, and a
green or red arrow flashes briefly.

**Pairings / Bracket** — the tee sheet grouped by tee time or shotgun hole, your own group
highlighted. In a bracket event, this tab shows the live bracket instead.

**More** — my full card, side contests, tournament info (course, format, rules as configured),
settings for appearance, and the admin panel if you hold the admin role.

**Admin panel** — tournament setup wizard (course, tees, date, format, handicap method),
player management, pairing generation, side contest configuration, a live score-editing grid,
and a lock-tournament action that freezes results.

---

## 9. Technical architecture

### Stack

| Layer | Choice | Why |
|---|---|---|
| Frontend | React + TypeScript, Vite | Fast, simple, static output. TypeScript because the scoring rules genuinely benefit from type safety. |
| Styling | Tailwind CSS with a custom Apple-derived token layer | Consistent spacing and color, no CSS sprawl. |
| Backend | Supabase (Postgres + Realtime + Row Level Security) | Real-time subscriptions over WebSocket out of the box, real relational integrity, generous free tier. Replaces writing a server entirely. |
| Local store | IndexedDB via Dexie | Offline source of truth. |
| Offline | Service worker + Workbox + Background Sync | Standard, well-trodden pattern. |
| Hosting | Cloudflare Pages (or the Synology NAS via the existing Portainer + GHCR pipeline) | Free, global, HTTPS by default. |

### Offline-first — the important part

The rule: **the local database is the source of truth for what you typed.** The network is a
sync channel, never a prerequisite.

```
  Tap a score
       ↓
  Write to IndexedDB immediately  ──→  UI updates instantly, always
       ↓
  Push onto a sync queue
       ↓
  Online?  ──no──→  wait; retry on the browser's online event
       │                 (Background Sync fires it even if the tab is closed)
      yes
       ↓
  POST to Supabase  ──fails──→  back on the queue, exponential backoff
       ↓
  Confirmed; reconcile local row, mark synced
```

The player never sees a spinner and never loses a score. The sync badge tells them the truth —
"3 holes waiting to sync" — without blocking anything. On reconnect, everything flushes and
the leaderboard catches up.

### Live updates

Supabase Realtime pushes Postgres changes to every subscribed client over a WebSocket. When a
scorer on hole 12 saves a birdie, every other phone sees the leaderboard reorder within about
a second, with no polling and no battery drain from a refresh loop.

Fallback chain, because golf course wifi is a rumor: WebSocket → if the socket drops, poll
every 30 seconds → if fully offline, show cached standings with a clear "as of 2:14 PM" stamp.
Never show stale data pretending to be live.

### Conflict resolution

Because scoring is normally one scorer per group, real conflicts are rare. The policy:

- Each score row is uniquely keyed by `(round, hole)`. Writes are upserts, not inserts, so a
  replayed offline queue is idempotent — syncing twice cannot double-count.
- If two devices edited the same hole while offline, last-write-wins by client timestamp, and
  the losing value is retained in an audit log.
- Any overwritten score raises a small conflict flag on the admin's score grid so a human can
  settle it. Silent data loss is not acceptable in something that decides who won.

### Security

Row Level Security policies enforce the role model at the database level, not just in the UI:

- Reading tournament data requires a valid token for that tournament.
- Writing a score requires the scorer role for that specific group, or the admin role.
- Admin-only tables (configuration, Peoria hole selection) are unreadable to players — which
  matters, since a leaked Peoria hole list defeats the entire system.

---

## 10. Data model

```
tournaments      id, name, code, admin_code_hash, date, status,
                 format, handicap_method, tiebreak_rules, settings

courses          id, name, par, holes
course_tees      id, course_id, name, rating, slope, color
holes            id, course_id, number, par, stroke_index, yardage[tee]

players          id, tournament_id, name, email, handicap_index,
                 course_handicap, tee_id, role, claimed_by_device

teams            id, tournament_id, name
team_members     team_id, player_id, position (A/B/C/D)

groups           id, tournament_id, tee_time, starting_hole, scorer_id
group_members    group_id, player_id

rounds           id, tournament_id, player_id | team_id, group_id, status
scores           id, round_id, hole_number, strokes, is_conceded,
                 updated_at, updated_by_device, synced_at
                 UNIQUE (round_id, hole_number)

matches          id, tournament_id, bracket_round, position,
                 player_a, player_b, winner, result_text, status

side_contests    id, tournament_id, type, hole_number, label
contest_entries  contest_id, player_id, value, photo_url, recorded_at

score_audit      id, score_id, old_value, new_value, changed_by, changed_at
```

Note that `scores` is keyed by hole *number*, not sequence — this is what makes shotgun starts
work correctly regardless of which hole a group began on.

---

## 11. Design system

The brief is "pretty like an Apple application," so this is treated as a real design layer, not
a coat of paint.

**Typography** — the system font stack (`-apple-system`), which resolves to SF Pro on every
Apple device. Type scale mirrors iOS: Large Title 34, Title 28, Headline 17 semibold, Body 17,
Footnote 13, Caption 12. Scores and totals use tabular figures so digits do not jitter as they
change.

**Color** — Apple's system palette, defined as CSS custom properties and flipped by a single
`data-theme` attribute on the root. Light is the default, per the brief, with an option to
follow the system setting.

| Token | Light | Dark |
|---|---|---|
| Background | `#F2F2F7` | `#000000` |
| Surface (cards) | `#FFFFFF` | `#1C1C1E` |
| Surface elevated | `#FFFFFF` | `#2C2C2E` |
| Separator | `#C6C6C8` | `#38383A` |
| Label | `#000000` | `#FFFFFF` |
| Label secondary | `#3C3C43` @ 60% | `#EBEBF5` @ 60% |
| Accent (fairway) | `#248A3D` | `#30D158` |
| Under par | `#248A3D` | `#30D158` |
| Over par | `#D70015` | `#FF453A` |
| Warning / offline | `#C93400` | `#FF9F0A` |

Score colors carry a shape as well as a hue — birdie in a circle, eagle in a double circle,
bogey in a square, exactly as a paper scorecard does. That keeps the leaderboard readable for
colorblind players rather than relying on red and green alone.

**Icons** — SF Symbols cannot be embedded on the web (Apple's license does not permit it), so
this ships as a hand-drawn monoline SVG set matching SF Symbols' geometry: 1.5–2pt rounded
strokes, `currentColor` fill, optically aligned on a 24×24 grid. The glyphs mirror their SF
counterparts one for one:

`figure.golf` · `list.bullet.rectangle` · `chart.bar.fill` · `person.2.fill` ·
`trophy.fill` · `flag.pattern.checkered` · `gearshape.fill` · `arrow.triangle.2.circlepath` ·
`wifi.slash` · `checkmark.seal.fill` · `plus.circle.fill` · `minus.circle.fill` ·
`ellipsis.circle`

If the app is later wrapped as a native iOS app, these swap for genuine SF Symbols with no
layout change.

**Materials and motion** — translucent blurred headers and tab bars over scrolling content
(`backdrop-filter`), Apple's standard corner radii (10 for controls, 14 for cards, 22 for
sheets), and spring-based transitions rather than linear easing. Leaderboard reordering
animates position rather than snapping. Every animation respects
`prefers-reduced-motion` and drops to a cross-fade.

**Touch and ergonomics** — 44×44pt minimum targets, primary actions in the lower third of the
screen where a thumb reaches, safe-area insets honored so nothing hides behind the home
indicator. Score steppers are oversized well beyond the minimum, because the real use case is a
gloved hand in direct sunlight.

**Accessibility** — WCAG AA contrast in both themes, full VoiceOver labeling, and support for
larger text sizes without layout breakage.

---

## 12. Build phases

Each phase ends at something usable. If time runs out mid-plan, whatever is finished still
works for an outing.

**Phase 1 — Foundation (week 1)**
Project scaffolding, TypeScript config, Tailwind with the full design token layer, light/dark
theming, the icon set, base components (list rows, cards, segmented control, tab bar), Supabase
project and schema, RLS policies.
*Ends with:* a themed, navigable shell.

**Phase 2 — Scoring engine (week 2)**
The pure scoring module with strategies for stroke play, Stableford, scramble, best ball,
shamble and match play. Handicap calculation, allowances, stroke allocation, Callaway and
Peoria. Comprehensive unit tests — this phase is tests-first, since the rules are known up
front and correctness here is everything.
*Ends with:* a proven engine, no UI.

**Phase 3 — Core loop (weeks 3–4) ← usable for a real outing here**
Tournament creation, code join, player claiming, group scorecard entry, the Play screen, and
the live leaderboard over Supabase Realtime.
*Ends with:* you could run an outing on this.

**Phase 4 — Offline (week 5)**
Service worker, IndexedDB local store, sync queue, Background Sync, conflict handling, sync
status UI, PWA manifest and home-screen install.
*Ends with:* it works in a dead zone, which is when it matters.

**Phase 5 — Pairings and brackets (week 6)**
Pairing generation across all four methods, drag-and-drop tee sheet, shotgun start support,
bracket generation and seeding, live bracket view with auto-advance, consolation bracket.

**Phase 6 — Extras (week 7)**
Skins, closest to the pin, longest drive. Score attestation. Position-change alerts. Admin
score grid and audit log.

**Phase 7 — Polish (week 8)**
Motion pass, accessibility audit, empty and error states, results sharing card, real-device
testing across iOS and Android, and a dry run on an actual course to see what breaks with real
signal.

---

## 13. Risks, and what we do about them

| Risk | Mitigation |
|---|---|
| No signal on the course | Offline-first from Phase 4. The whole architecture assumes this. |
| Scorer's phone dies | Admin can promote any player in the group to scorer instantly. Scores already synced are safe. |
| Someone taps the wrong name on join | Admin can unclaim and reassign. |
| Scoring disputes | Every change is audited with who and when. Attestation flow makes the card explicit. |
| A format we did not implement | The strategy pattern makes adding one a contained change. Ask early which formats you actually play. |
| Course rating / slope data | Entered by hand at setup, once per course. Two minutes off the back of the scorecard. Courses are reusable across tournaments. |
| Wrong stroke indexes | Same — typed once from the scorecard, then stored. |

---

## 14. Decisions I need from you

None of these block starting, but each one shapes the work:

1. **Which formats do you actually play?** If it is always a four-person scramble, Phase 2
   shrinks by half and we get to a usable app faster. I have planned for the full set.
2. **Shotgun start or tee times?** Both are supported, but if you only ever do one, the other
   can be deferred.
3. **Do you want net scoring at all?** If everyone plays gross, the entire handicap system in
   §7 becomes optional and Phase 2 gets considerably lighter.
4. **Multi-round or single-day?** The schema supports multiple rounds per tournament; the UI is
   planned for single-day. Multi-round adds a round selector nearly everywhere.
5. **Hosting:** Cloudflare Pages, or your Synology NAS through the existing Portainer and GHCR
   pipeline? Cloudflare is simpler and more reliable from a golf course; the NAS keeps
   everything under your own roof.
6. **Does this need to work beyond staff?** If it might be used for a church-wide charity
   scramble with 100+ players, a few decisions (pairing UI, leaderboard virtualization) change
   now rather than later.

---

## 15. Recommendation

Build it. Start with Phases 1–3 and get something you can use at the next outing, then add
offline and brackets once you have seen it fail in a real parking lot — because it will, and
those failures are the best spec you will get.

The highest-value thing to lock down before writing code is question 1 above. Everything else
can be decided as we go.

---

## Sources

- [Golf Tournament Formats Explained: Scramble, Shamble, Best Ball — Kismet Golf](https://kismet.golf/post/golf-tournament-format)
- [Golf Shamble Format: Rules, Scoring & Strategy — Golf Decode](https://golfdecode.com/golf-shamble-format/)
- [Competition Formats: Stableford — HowDidiDo](https://howdidido.blob.core.windows.net/clubsitespublic/file_2a2e480b-46f6-452d-ac67-caaab2d3d760.pdf)
- [Popular Formats — Elite Golf Management](https://elitegolf.co/popular-formats/)
- [Live Leaderboards — Golf Genius](https://golfgenius.com/resources/live)
- [Leaderboards — 18Birdies Knowledge Base](https://help.18birdies.com/article/624-leaderboards)
- [Scoring formats & side games — Golf Pad Support](https://support.golfpadgps.com/support/solutions/articles/6000235479-scoring-formats-side-games-available-in-golf-pad-events)
- [Squabbit — Free Golf Tournament & League Software](https://squabbitgolf.com/)
- [VPAR Side Games](https://vpar.com/vpar-side-games/)
- [Peoria Handicap System: How to Score a Golf Outing Without Handicaps — Golf Games Hub](https://www.golfgameshub.com/peoria-handicap-system/)
- [Using the Callaway System Method for a 1-Day Golf Handicap — LiveAbout](https://www.liveabout.com/how-to-use-the-callaway-system-1564469)
- [Appendix C — Handicap Allowances (WHS)](https://glenviewseniormens.golfclub.net/cm/80390/uploads/Handicap%20Allowances.pdf)
- [Tournament Formats and Allowances — SCGA](https://www.scga.org/scga-blog2/view/tournament-formats-and-allowances)
- [Offline-First PWA Patterns — Service Workers, IndexedDB, and Background Sync](https://rohitraj.tech/en/notes/pwa-offline-sync)
- [Building an Offline-First PWA with Next.js, IndexedDB, and Supabase](https://oluwadaprof.medium.com/building-an-offline-first-pwa-notes-app-with-next-js-indexeddb-and-supabase-f861aa3a06f9)
