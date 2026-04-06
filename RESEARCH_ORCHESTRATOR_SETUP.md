# Research Orchestrator Setup & Usage

## What You Now Have

A cost-optimized skill that orchestrates Claude → Ollama → Claude → GSD workflow with **full token cost tracking**.

**Your goal:** Save on Claude API costs by doing research with free local Ollama, while keeping Claude in control.

**The solution:** This skill manages the handoff and shows you exactly what's free vs. what costs.

---

## The Workflow

```
You: /research-orchestrator "I want to build X"
       ↓
Claude: "Run this in Ollama terminal:" [~200 tokens cost]
       ↓
You: Run in Ollama (separate terminal window)
       ↓
You: Paste response back to Claude
       ↓
Claude: Summarizes + shows cost [~300 tokens cost]
       ↓
You: Decide: continue research / move to GSD / quick follow-up
       ↓
If continue: Back to step 2 (loop is FREE research + cheap summary)
If GSD: Claude generates GSD context [~400 tokens cost]
       ↓
You: Use context with /gsd-new-project
       ↓
Continue with GSD planning/execution
```

---

## How to Use

### Step 1: Start Research

```
/research-orchestrator "Build a real-time metrics dashboard"
```

Claude will show:
```
✅ RESEARCH ORCHESTRATION STARTED

[COST: ~200 tokens]
← Claude generating optimized research prompt

---

OLLAMA RESEARCH PROMPT:

Copy and paste this into Ollama terminal:

  ollama run mistral "Analyze real-time metrics dashboard 
  architectures. Compare: WebSocket patterns, state management,
  caching layers, database options..."

This takes ~2-3 seconds.

NEXT:
Run the command above, then paste Ollama's response here.
```

### Step 2: Run in Ollama Terminal

Open a separate terminal and run the exact command Claude gives you:

```bash
ollama run mistral "Analyze real-time metrics dashboard..."
```

Takes 2-3 seconds, costs $0.

### Step 3: Paste Response Back

Copy Ollama's response and paste it back in Claude Code.

Claude will:
```
[COST: ~300 tokens]
← Claude processing Ollama response + summarizing

---

✅ RESEARCH SUMMARY

KEY FINDINGS:
1. WebSockets recommended over polling (lower latency)
2. State management: Zustand better than Redux for this use case
3. Caching: Redis layer for backend

CONFIDENCE: High

COST SUMMARY SO FAR:
Research: FREE (Ollama)
Processing: ~500 tokens
Without Ollama would cost: ~5000 tokens
Savings: ~90%

---

NEXT DECISION:

Option A: Continue research
[FREE - Ollama will analyze]
"Continue research: state management deep dive"

Option B: Move to GSD
[COST - Generate context]
"Move to GSD"

Option C: Quick follow-up
[FREE + COST]
"Quick follow-up: specific pattern question"

What next?
```

### Step 4: Decide What's Next

**Option A: Continue Research**
```
"Continue research: deep dive into Zustand vs Redux for this case"
```
→ Another FREE research round with Ollama + cheap summary (total ~500 tokens for round)

**Option B: Move to GSD**
```
"Move to GSD"
```
→ Claude generates GSD-ready context (~400 tokens)

**Option C: Quick Follow-up**
```
"Quick follow-up: specific pattern for metrics updates?"
```
→ Another FREE research round with specific question

### Step 5: Use with GSD (When Ready)

When you have enough research, Claude will show:

```
✅ READY FOR GSD PLANNING

GSD CONTEXT (copy this):

"Real-time metrics dashboard. Architecture: WebSockets + Zustand 
+ Redis caching. Key decisions: [from research]. Key risks: [from 
research]."

---

SUGGESTED COMMAND:

/gsd-new-project Real-Time Metrics Dashboard

When prompted for context, paste the GSD CONTEXT above.

---

COST BREAKDOWN:
Research phase: ~1,500 tokens
Without Ollama: ~10,000 tokens
Savings: ~85%
```

Then run:
```
/gsd-new-project Real-Time Metrics Dashboard
```

Paste the research context when prompted.

---

## Cost Display Format

Every response shows costs clearly:

```
[FREE]               ← Ollama research (you run locally)
[COST: ~200 tokens]  ← Claude generating prompts
[COST: ~300 tokens]  ← Claude summarizing
[COST SUMMARY]       ← Total costs and savings breakdown
```

This lets you see:
- ✅ What's free (Ollama research)
- ✅ What costs (Claude processing)
- ✅ Total spending
- ✅ What you saved (vs. direct Claude)

---

## Real-World Example

**Scenario: Build OAuth2 Payment System**

```
You: /research-orchestrator "Build OAuth2 with payments"

Claude: [COST: ~200 tokens]
        Generate research prompt

You: Run in Ollama (2 seconds, FREE)

You: Paste response

Claude: [COST: ~300 tokens]
        RESEARCH SUMMARY:
        - Stripe for payments
        - Auth0 for OAuth
        - Reasons: cost, scalability, support
        
        COST SO FAR: ~500 tokens
        Without Ollama: ~4000 tokens

You: "Continue research: deployment strategy"

Claude: [COST: ~200 tokens]
        Generate new prompt

You: Run in Ollama (2 seconds, FREE)

You: Paste response

Claude: [COST: ~350 tokens]
        RESEARCH SUMMARY - UPDATED:
        - Payment: Stripe + webhooks
        - Auth: Auth0 + refresh tokens
        - Deployment: Kubernetes
        
        COST SO FAR: ~1050 tokens
        Without Ollama: ~8000 tokens
        SAVINGS: ~87%

You: "Move to GSD"

Claude: [COST: ~400 tokens]
        GSD CONTEXT generated
        
        TOTAL RESEARCH COST: ~1,450 tokens
        WITHOUT OLLAMA: ~12,000 tokens
        PROJECT SAVINGS: ~88%

You: /gsd-new-project OAuth2 Payment System
     [paste context above]
```

**Cost breakdown:**
- Research phase (what you just did): ~1,450 tokens
- GSD planning (next): ~5,000 tokens
- GSD execution (next): ~20,000 tokens
- **Total: ~26,450 tokens**
- Without Ollama would be: ~40,000 tokens
- **You saved: ~34% on the entire project**

---

## Key Insight: Multiple Research Rounds Are Cheap

Because research is free with Ollama, you can afford to explore multiple angles:

```
Round 1 research: FREE (Ollama) + $0.006 (summary)
Round 2 research: FREE (Ollama) + $0.006 (summary)
Round 3 research: FREE (Ollama) + $0.010 (integrated summary)

Total: ~1,500 tokens (~$0.045)

Direct Claude approach:
Round 1: ~3,500 tokens
Round 2: ~3,500 tokens
Round 3: ~3,500 tokens
Total: ~10,500 tokens (~$0.315)

Difference: ~$0.27 per project for 3 research rounds
Over 100 projects: ~$27 saved just on research
```

**Because it's so cheap, you can research more thoroughly = better decisions.**

---

## Files You Can Reference

All created in `.claude/skills/`:

1. **`research-orchestrator.md`** — Full documentation with detailed examples
2. **`research-orchestrator-workflow.md`** — How Claude executes the skill
3. **`RESEARCH_ORCHESTRATOR_README.md`** — Quick reference + cost tables
4. **`RESEARCH_ORCHESTRATOR_SETUP.md`** — This file

---

## Usage Tips

### Maximize Efficiency

1. **Use multiple research rounds** (it's cheap!)
   - Each round: ~500 tokens (vs ~3,500 without Ollama)
   - Multiple perspectives = better decisions

2. **Ask specific Ollama questions**
   - Claude generates the prompts
   - More specific → better answers → less iteration

3. **Save research summaries**
   - Copy the RESEARCH SUMMARY section
   - Keep in project notes for reference

4. **Feed GSD the research context**
   - Use the context Claude generates
   - Plans will be better informed

5. **Track your savings over time**
   - Keep the cost breakdowns
   - Look for patterns
   - Find where research saves most

### When to Use

✅ **Use Research Orchestrator when:**
- Starting a new project
- Architecture is uncertain
- Multiple approaches exist
- Want validated decisions before planning
- Building complex systems

❌ **Skip when:**
- Simple feature on existing project
- Architecture already decided
- Quick bug fix
- Not worth 5 minutes of research

---

## Cost Expectations

**Per-project research phase:**
- Single round: ~500 tokens (~$0.015)
- Three rounds: ~1,500 tokens (~$0.045)
- Five rounds: ~2,500 tokens (~$0.075)

**Without Ollama (hypothetical):**
- Single round: ~3,500 tokens (~$0.105)
- Three rounds: ~10,000 tokens (~$0.30)
- Five rounds: ~15,000 tokens (~$0.45)

**You save:** ~85% on research phase costs with Ollama

---

## Quick Reference Commands

```
Start research:
  /research-orchestrator "Your project description"

Continue exploring (FREE research + cheap summary):
  "Continue research: different angle"

Quick answer (FREE research):
  "Quick follow-up: specific question"

Move to planning:
  "Move to GSD"

Then use GSD:
  /gsd-new-project [Project Name]
  [paste context Claude provided]
```

---

## What Happens Behind the Scenes

**When you call `/research-orchestrator`:**

1. Claude reads your project scope
2. Claude identifies key research questions
3. Claude generates an optimized Ollama prompt
4. Claude tells you exactly what to run
5. Claude tracks the cost of this generation (~200 tokens)

**When you run in Ollama:**

1. Ollama analyzes your question
2. Ollama returns detailed analysis
3. Cost: $0 (local hardware)
4. You paste response back to Claude

**When you paste the response:**

1. Claude processes the Ollama analysis
2. Claude extracts key findings
3. Claude integrates with previous research (if multiple rounds)
4. Claude summarizes for you and for Ollama (if you continue)
5. Claude shows cost of processing (~300 tokens)
6. Claude shows cumulative savings

**If you continue researching:**

1. Back to step 1 (new angle)
2. Ollama research is free
3. Claude summary is cheap
4. You get more thorough understanding

**When you move to GSD:**

1. Claude synthesizes all research findings
2. Claude generates GSD-ready context
3. Claude costs ~400 tokens for this
4. You copy/paste into `/gsd-new-project`
5. GSD planning starts with validated context

---

## Start Now

You're ready to use this. Next time you want to start a project:

```
/research-orchestrator "Your project idea"
```

Claude will guide you through the rest.

**Every step will show:** FREE or COST so you know exactly what you're spending.

Good luck saving on your Claude bill! 🚀
