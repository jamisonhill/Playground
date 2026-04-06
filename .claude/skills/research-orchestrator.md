# Research Orchestrator Skill

**Purpose:** Efficiently coordinate Claude → Ollama → Claude → GSD workflow with full cost transparency

## How It Works

This skill manages a loop where:
1. **Claude generates research prompts** [COST: ~200 tokens]
2. **You run in Ollama** [FREE]
3. **Claude summarizes findings** [COST: ~300 tokens]
4. **Loop for deeper research** [FREE → COST → FREE]
5. **Integrate with GSD** [COST: ongoing]

Each step is marked with cost so you can track efficiency.

---

## Usage

### Initial Research

```
You: /research-orchestrator "I want to build a real-time 
dashboard for tracking metrics across 5 backend services"
```

Claude (via this skill) will:
1. Generate optimized Ollama research prompt
2. Show cost estimate
3. Tell you what to paste in Ollama
4. Wait for your response

### During Research Loop

```
You: [paste Ollama response here]
```

Claude will:
1. Summarize findings [COST]
2. Show accumulated research
3. Offer: continue research / move to GSD / quick follow-up

### Moving to GSD

When you have enough research, Claude will:
1. Generate GSD-ready context
2. Show full cost breakdown
3. Suggest `/gsd-new-project` command

---

## Cost Display Format

Each response shows:

```
[COST: ~400 tokens]
← Claude processing + summarization

[FREE]
← Ollama research (you run in terminal)

[COST SUMMARY]
Research phase: 800 tokens
Would have cost: ~3000 tokens without Ollama
Savings: ~73%
```

This helps you track efficiency and identify optimization opportunities.

---

## The Research Loop

```
START: You provide project scope
  ↓
[COST: ~200 tokens]
Claude generates research prompt
  ↓
Returns: Exact text to paste in Ollama
  ↓
[FREE]
You: Run in Ollama terminal
ollama run mistral "[prompt from Claude]"
  ↓
You: Paste response back to Claude
  ↓
[COST: ~300 tokens]
Claude summarizes + extracts key findings
  ↓
DECISION POINT:
  → Continue research? [FREE again]
  → Move to GSD? [Show context]
  → Quick follow-up question? [Show Ollama prompt]
  → Done? [Show cost summary]
```

---

## Example Interaction

### You: `/research-orchestrator "Real-time metrics dashboard"`

**Claude responds:**

```
✅ RESEARCH ORCHESTRATION STARTED

PROJECT: Real-time metrics dashboard

RESEARCH PLAN:
1. Architectural patterns for real-time systems
2. State management approaches
3. Data flow and performance considerations

---

STEP 1: Generate Ollama Research Prompt

[COST: ~200 tokens]
← Claude generating optimized research prompt

---

OLLAMA RESEARCH PROMPT:

Copy and paste this into Ollama terminal:

  ollama run mistral "Analyze real-time metrics dashboard 
  architectures. Compare: WebSocket + state management 
  patterns, polling strategies, caching layers, database 
  considerations for 100+ metrics. What's the recommended 
  approach and why?"

This will take ~2-3 seconds.

---

NEXT:
Run the command above, then paste Ollama's response here.
Say "continue research" if you want to explore more angles.
Say "move to gsd" when you're ready for planning.
```

### You: [paste Ollama response]

**Claude responds:**

```
[COST: ~300 tokens]
← Claude processing Ollama response + summarizing

---

✅ RESEARCH SUMMARY - REAL-TIME METRICS DASHBOARD

KEY FINDINGS:
1. Architecture: WebSockets + Zustand recommended
   - WebSockets for real-time updates (lower latency than polling)
   - Zustand for state (lightweight, better than Context for high-frequency updates)
   - Redis caching layer for backend

2. Performance considerations:
   - Batch updates every 500ms to avoid network saturation
   - Virtual scrolling for large metric lists
   - Selective subscriptions (only metrics user is viewing)

3. Database approach:
   - TimescaleDB or similar for metrics (optimized for time-series)
   - Separate from main app database

CONFIDENCE: High (Ollama reasoning aligned on all points)

---

COST SUMMARY SO FAR:
- Research step 1: FREE (Ollama)
- Processing step 1: ~300 tokens
- Total cost: ~300 tokens
- Equivalent without Ollama: ~4000 tokens
- Savings: ~92%

---

NEXT DECISION:

Option A: Continue research
[FREE - Ollama will analyze next angle]
"Continue research: deep dive into Zustand vs Redux"

Option B: Move to GSD Planning
[COST - Claude planning phase begins]
"Move to GSD"

Option C: Quick follow-up question
[FREE - Ollama answer + COST - Claude processing]
"Quick follow-up: specific Zustand patterns for this"

What would you like to do?
```

### You: "Continue research: deep dive into Zustand vs Redux"

**Claude responds:**

```
✅ RESEARCH LOOP - Round 2

[COST: ~200 tokens]
← Claude refining research based on first findings

---

REFINED OLLAMA RESEARCH PROMPT:

ollama run mistral "Given a real-time metrics dashboard 
with 100+ metrics updating every 500ms: Compare Zustand 
vs Redux. Include: performance characteristics, developer 
experience, testing approach, learning curve, ecosystem, 
bundle size, middleware options. Recommend one with clear 
reasoning."

---

Run this, then paste Ollama's response.
```

### You: [paste second Ollama response]

**Claude responds:**

```
[COST: ~350 tokens]
← Claude integrating two research perspectives

---

✅ RESEARCH SUMMARY - UPDATED

KEY FINDINGS (CONSOLIDATED):

Architecture Stack:
  Frontend: React + Zustand (recommended over Redux)
  Real-Time: WebSockets + batching
  Caching: Redis for backend
  Database: TimescaleDB for metrics
  
Decision: Zustand Rationale:
  ✅ 73% smaller than Redux (2.3KB vs 7KB gzipped)
  ✅ Better performance for high-frequency updates
  ✅ Simpler API reduces learning curve
  ✅ DevTools support sufficient
  ⚠️ Smaller ecosystem (but mature enough for this use case)

Implementation Approach:
  1. Zustand store for metric subscriptions
  2. WebSocket handler with 500ms batching
  3. Virtual scrolling for large lists
  4. Redis caching layer on backend

Risks & Mitigations:
  - Risk: Zustand immutability with large datasets
    Mitigation: Normalize store shape, use selectors
  - Risk: WebSocket scaling
    Mitigation: Use message queues (RabbitMQ) if >1000 connections

---

COST SUMMARY - COMPLETE RESEARCH:
- Research round 1: FREE (Ollama)
- Processing round 1: ~300 tokens
- Research round 2: FREE (Ollama)
- Processing round 2: ~350 tokens
- Total research cost: ~650 tokens
- Equivalent without Ollama: ~7000 tokens
- Savings: ~91%

RESEARCH CONFIDENCE: Very High (two independent analyses aligned)

---

READY FOR GSD?

You now have validated architectural decisions based on free Ollama 
research. Ready to move to GSD planning?

Say "Move to GSD" or ask another follow-up question.
```

### You: "Move to GSD"

**Claude responds:**

```
✅ READY FOR GSD PLANNING

[COST: ~400 tokens]
← Claude generating GSD-ready context

---

GSD CONTEXT (for /gsd-new-project):

Copy this as context when you run /gsd-new-project:

PROJECT CONTEXT:
"Real-time metrics dashboard for 5 backend services. 
Architecture: React + Zustand + WebSockets + TimescaleDB.
Key decisions: Zustand over Redux (smaller, better perf), 
WebSockets with 500ms batching, Redis caching, virtual 
scrolling for large lists. Key risk: Zustand immutability 
with large datasets (mitigation: normalized store, selectors)."

---

SUGGESTED GSD COMMAND:

/gsd-new-project Real-Time Metrics Dashboard

When prompted for context, paste the GSD CONTEXT above.

---

COMPLETE COST BREAKDOWN:

Research Phase (Ollama): FREE
  - 2 research rounds, 2 analyses

Processing Phase (Claude): ~650 tokens
  - 1 initial prompt: ~200 tokens
  - 1 summary round 1: ~300 tokens
  - 1 summary round 2: ~350 tokens
  - 1 GSD context: ~400 tokens
  - Total so far: ~1,250 tokens

Without Ollama (hypothetical):
  - Would have cost: ~8,000 tokens
  - You saved: ~84% on research

GSD Planning Phase (upcoming):
  - Will cost: ~3,000-5,000 tokens depending on complexity
  - Base cost (unavoidable)

GSD Execution Phase (upcoming):
  - Will cost: ~15,000-20,000 tokens depending on scope
  - Base cost (unavoidable)

TOTAL EFFICIENCY:
- With Ollama research: ~20,000 tokens total project
- Without Ollama: ~28,000 tokens total project
- Project savings: ~28%

---

Next step: Run /gsd-new-project as shown above.
Good luck! 🚀
```

---

## Cost Tracking Summary

Every response shows costs:

```
[FREE]               ← Ollama running (you pay electricity)
[COST: ~200 tokens]  ← Claude processing (you pay API)
[COST SUMMARY]       ← Breakdown of efficiency
```

This helps you:
- ✅ See real-time costs vs free operations
- ✅ Identify where money is being spent
- ✅ Find optimization opportunities
- ✅ Understand actual savings with Ollama

---

## Key Features

### 1. **Smart Research Generation**
Claude generates research prompts optimized for Ollama (not verbose, specific questions)

### 2. **Multi-Round Research**
Keep researching different angles, all free, until you're confident

### 3. **Automatic Summarization**
Each Ollama response is summarized and integrated (no token waste)

### 4. **GSD Integration**
Generated context fits directly into `/gsd-new-project`

### 5. **Cost Transparency**
Every step marked as FREE or COST so you see efficiency

### 6. **Consolidated Findings**
Multiple research rounds are intelligently combined

---

## When to Use Each Option

### Continue Research
Use when:
- ✅ Different aspect you want to explore
- ✅ Need deeper analysis of one angle
- ✅ Want multiple perspectives before deciding

Cost: FREE (Ollama runs) → COST (Claude summarizes)

### Move to GSD
Use when:
- ✅ Architecture is validated
- ✅ Key decisions are made
- ✅ Ready for planning phase

Cost: One final COST step (GSD context generation)

### Quick Follow-up
Use when:
- ✅ Clarifying specific point from research
- ✅ One more quick answer needed
- ✅ Mid-implementation question during coding

Cost: FREE (Ollama) → COST (Claude processing)

---

## Pro Tips

1. **Ask specific questions in Ollama prompts**
   - Claude generates prompts with clear criteria
   - Ollama gives better answers to specific questions

2. **Use multiple rounds for confidence**
   - Ask same question different ways (free!)
   - Validates findings before moving to paid planning

3. **Save research summaries to project notes**
   - Copy the RESEARCH SUMMARY section
   - Refer back if planning changes direction

4. **Track savings over time**
   - Save cost summaries
   - Look for patterns in what saves most

5. **Combine with GSD efficiently**
   - Feed research context to `/gsd-new-project`
   - Reduces planning phase token cost too

---

## What Gets Tracked

**FREE Operations:**
- All Ollama research (you pay electricity, not API)
- Your reading/copying time

**COST Operations:**
- Claude generating research prompts (~200 tokens)
- Claude summarizing Ollama responses (~300 tokens each)
- Claude generating GSD context (~400 tokens)
- GSD planning phase (unavoidable)
- GSD execution phase (unavoidable)

**Optimization:**
- Multiple research rounds → minimal cost increase
- Ollama research → huge token savings
- Consolidated summaries → no wasted tokens

---

## Example Savings Calculation

**Your project: Build payment system**

Without Ollama (direct Claude):
```
Requirements gathering: 4000 tokens
Architecture exploration: 6000 tokens
Planning: 8000 tokens
Implementation: 20000 tokens
Total: 38000 tokens (~$1.14)
```

With Ollama orchestrator:
```
Research (FREE Ollama): 0 tokens
Summarization: 600 tokens
GSD Planning: 5000 tokens
Implementation: 20000 tokens
Total: 25600 tokens (~$0.77)
Savings: ~33%
```

Over 100 projects: **~$37 saved per year** just on research phase.

---

## Start Using

Call it:

```
/research-orchestrator "Your project scope here"
```

Claude will take it from there with full cost transparency.

Every step, you'll know if it's FREE (Ollama) or COST (Claude).
