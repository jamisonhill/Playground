# Healthy Breakfast Website

## What This Is

A local Vue.js learning project built as an interactive healthy breakfast recipe website. Displays recipes with nutrition info, search, and filtering — hosted locally on macOS. Primary goal: learn Vue.js fundamentals through a real, useful application.

## Core Value

Learn Vue.js through building an interactive, functional web application.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] **SETUP-01**: Vue CLI project initialized with Express.js backend
- [ ] **DATA-01**: Recipe data structure in JSON (name, ingredients, nutrition, prep_time)
- [ ] **DISPLAY-01**: Display recipe cards with all information
- [ ] **DISPLAY-02**: Show nutrition breakdown (calories, protein, carbs, fat)
- [ ] **SEARCH-01**: Search recipes by name
- [ ] **SEARCH-02**: Filter by prep time or nutrition criteria
- [ ] **UI-01**: Mobile-responsive layout
- [ ] **HOSTING-01**: Express.js serves Vue app on localhost

### Out of Scope

- Database (JSON file storage for v1) — simplifies learning, defer SQL to v2
- User authentication — single-user local app
- Favorites persistence (beyond session) — sessions sufficient for learning
- Advanced styling/animations — focus is Vue, not CSS
- Admin interface — manually edit JSON data for now

## Context

**From Research (Ollama):**
Vue.js v3 recommended for ease of learning and flexibility. Express.js + JSON data chosen to avoid database complexity. Interactivity via computed properties, watchers, and methods. Moderate learning curve (expected 5-6 phases to complete).

**Why This Matters:**
Learning Vue in context of a real feature (recipe discovery) makes fundamentals stick better than isolated tutorials.

## Constraints

- **Tech Stack**: Vue.js v3 + Express.js + JSON — non-negotiable per research
- **Hosting**: Local only (macOS localhost) — no cloud deployment
- **Data**: Start with 10+ recipes in JSON — expandable later
- **Timeline**: MVP-focused, 5-6 phases estimated
- **Scope**: Recipes/search/filters only; no user accounts

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Vue.js over React | Easier learning curve, simpler for MVP | — Pending |
| JSON over database | Reduces complexity, focuses on Vue learning | — Pending |
| Express.js server | Straightforward local hosting, Node.js familiarity | — Pending |
| MVP scope (search only) | Learn core Vue patterns before advanced features | — Pending |

---
*Last updated: 2026-04-06 after questioning phase*
