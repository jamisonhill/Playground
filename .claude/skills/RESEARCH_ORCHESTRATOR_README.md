# Research Orchestrator Skill - Quick Reference

## What It Does

Manages intelligent handoff between Claude and Ollama for cost-efficient research:

```
Claude generates prompt → You run in Ollama → Claude summarizes
      [COST]                   [FREE]              [COST]
              ↓
        [LOOP if needed]
              ↓
        Generate GSD context
              ↓
        /gsd-new-project
```

## How to Use

### Start Research

```
/research-orchestrator "Build a real-time metrics dashboard"
```

Claude will:
1. Generate research prompt to paste in Ollama
2. Show cost: `[COST: ~200 tokens]`
3. Tell you exactly what to run
4. Wait for your response

### Run in Ollama Terminal

```bash
# In separate terminal, Claude gives you this command
ollama run mistral "Analyze real-time metrics dashboard..."
```

Takes 2-3 seconds, costs $0.

### Paste Response Back to Claude

You paste Ollama's response back in Claude Code.

Claude will:
1. Process it: `[COST: ~300 tokens]`
2. Summarize findings
3. Show cost so far
4. Offer next step

### Decide Next Step

**Option A: Continue Research**
```
"Continue research: compare state management options"
```
→ Another FREE round with Ollama

**Option B: Move to GSD**
```
"Move to GSD"
```
→ Claude generates GSD-ready context

**Option C: Quick Follow-up**
```
"Quick follow-up: specific pattern for this?"
```
→ Another FREE round with Ollama

## Cost Transparency

Every step is marked:

```
[FREE]               ← Ollama (you run locally, $0)
[COST: ~200 tokens]  ← Claude (you pay API, ~$0.006)
[COST: ~300 tokens]  ← Claude processing response
[COST SUMMARY]       ← Total cost and savings shown
```

This lets you see:
- ✅ What's free (Ollama research)
- ✅ What costs (Claude processing)
- ✅ How much you saved (vs. direct Claude)

## Real Examples

### Example 1: Payment System

```
You: /research-orchestrator "Build OAuth2 payment system"

Claude: [COST: ~200 tokens]
        Generate research prompt...
        
        Run this:
        ollama run mistral "Compare payment processors...
        
You: [paste Ollama response]

Claude: [COST: ~300 tokens]
        RESEARCH SUMMARY:
        
        KEY FINDINGS:
        1. Recommended: Stripe for payments, Auth0 for OAuth
        2. Rationale: [consolidated from Ollama]
        
        COST SO FAR: ~500 tokens
        Would have cost without Ollama: ~5000 tokens
        Savings: ~90%

You: Continue research: infrastructure deployment

Claude: [COST: ~200 tokens]
        Generate refined prompt...
        
        Run this:
        ollama run mistral "Analyze deployment patterns..."
        
You: [paste response]

Claude: [COST: ~300 tokens]
        RESEARCH SUMMARY - UPDATED
        
        KEY FINDINGS (CONSOLIDATED):
        1. Payment: Stripe + webhooks
        2. Auth: Auth0 + refresh tokens
        3. Deployment: Kubernetes with auto-scaling
        
        COST SO FAR: ~800 tokens
        Would have cost without Ollama: ~8000 tokens
        Savings: ~90%

You: Move to GSD

Claude: [COST: ~400 tokens]
        Ready for GSD Planning
        
        SUGGESTED COMMAND:
        /gsd-new-project Payment System v1
        
        TOTAL PROJECT EFFICIENCY:
        Research phase: ~800 tokens
        Without Ollama: ~8000 tokens
        Savings: ~90% on research
```

### Example 2: Real-Time Dashboard (Multiple Rounds)

```
Round 1:
  You: /research-orchestrator "Real-time metrics dashboard"
  Claude: [COST: ~200] Generate prompt
  You: Run in Ollama (FREE)
  Claude: [COST: ~300] Summary
  
Round 2:
  You: "Continue research: state management options"
  Claude: [COST: ~200] Generate prompt
  You: Run in Ollama (FREE)
  Claude: [COST: ~300] Summary + integrated findings
  
Round 3:
  You: "Continue research: performance optimization"
  Claude: [COST: ~200] Generate prompt
  You: Run in Ollama (FREE)
  Claude: [COST: ~350] Summary + all three rounds consolidated
  
TOTAL COST: ~1,250 tokens
WITHOUT OLLAMA: ~10,000 tokens
SAVINGS: ~87%
```

## Key Insight: Multiple Rounds Are CHEAP

```
Round 1: FREE (Ollama) + COST (Claude) = ~500 tokens
Round 2: FREE (Ollama) + COST (Claude) = ~500 tokens
Round 3: FREE (Ollama) + COST (Claude) = ~550 tokens
Total:   ~1,550 tokens

WITHOUT OLLAMA:
Round 1 (Claude only):   ~3500 tokens
Round 2 (Claude only):   ~3500 tokens
Round 3 (Claude only):   ~3500 tokens
Total:   ~10,500 tokens

SAVINGS: ~85%
```

**Because research is free with Ollama, you can do 3-4 rounds for less than Claude would cost for 1 round.**

This means: **More research → Better decisions → Worth the small Claude cost**

## Cost Breakdown Examples

### Small Project

```
Without Ollama:
- Requirements: 4,000 tokens
- Planning: 8,000 tokens
- Implementation: 20,000 tokens
TOTAL: 32,000 tokens (~$0.96)

With Ollama Research:
- Ollama research: 0 tokens (FREE)
- Claude summarization: 800 tokens
- Planning: 6,000 tokens (cheaper with research)
- Implementation: 20,000 tokens
TOTAL: 26,800 tokens (~$0.80)
SAVINGS: ~16%
```

### Medium Project

```
Without Ollama:
- Requirements: 6,000 tokens
- Architecture: 10,000 tokens
- Planning: 10,000 tokens
- Implementation: 25,000 tokens
TOTAL: 51,000 tokens (~$1.53)

With Ollama Research:
- Ollama research: 0 tokens (FREE)
- Claude summarization: 1,200 tokens (3 rounds)
- Planning: 7,000 tokens (cheaper with validated arch)
- Implementation: 25,000 tokens
TOTAL: 33,200 tokens (~$1.00)
SAVINGS: ~35%
```

### Large Project

```
Without Ollama:
- Requirements: 8,000 tokens
- Architecture: 15,000 tokens
- Design patterns: 12,000 tokens
- Planning: 12,000 tokens
- Implementation: 35,000 tokens
TOTAL: 82,000 tokens (~$2.46)

With Ollama Research:
- Ollama research: 0 tokens (FREE, 4 rounds)
- Claude summarization: 1,800 tokens
- Planning: 8,000 tokens (cheaper with validated decisions)
- Implementation: 35,000 tokens
TOTAL: 44,800 tokens (~$1.34)
SAVINGS: ~45%
```

## When to Use Research Orchestrator

✅ **Use when:**
- Starting a new project
- Architecture is uncertain
- Multiple approaches exist
- Want validated decisions before planning
- Building complex systems

❌ **Skip when:**
- Simple feature on existing project
- Architecture already decided
- Quick bug fix or small task
- Not worth 5 minutes of research time

## Workflow Summary

```
1. /research-orchestrator [project]
   Claude generates Ollama prompt [COST: ~200]
   
2. You run in Ollama terminal
   Ollama returns analysis [FREE]
   
3. Paste response back
   Claude summarizes [COST: ~300]
   
4. Repeat 2-3 as needed (each round adds ~500 tokens)
   
5. "Move to GSD"
   Claude generates GSD context [COST: ~400]
   
6. /gsd-new-project [with context from above]
   GSD planning begins with researched foundation
   
7. /gsd-execute-phase
   Implementation with confidence
```

## Files

- `research-orchestrator.md` — Full documentation with examples
- `research-orchestrator-workflow.md` — Execution workflow details
- This file — Quick reference and cost examples

## Start Now

```
/research-orchestrator "Your project idea"
```

Claude will guide you through the rest.

Every step is marked FREE or COST so you know exactly what you're spending.

---

## Pro Tips for Maximum Efficiency

1. **Research before planning**
   - FREE research rounds → cheaper planning
   - ~5 minutes of research saves ~2000 tokens in planning

2. **Ask specific Ollama questions**
   - Not: "Tell me about databases"
   - But: "Compare PostgreSQL vs MongoDB for 1M+ documents with 10k ops/sec"
   - Better questions → better answers → less iteration

3. **Use multiple rounds**
   - Each round is nearly free (only Claude summarizes)
   - Multiple perspectives → better decisions
   - Worth the small Claude cost

4. **Save research summaries**
   - Copy the final RESEARCH SUMMARY
   - Keep in project notes
   - Refer back if priorities shift

5. **Feed GSD the research context**
   - Use the GSD CONTEXT Claude generates
   - Paste when creating /gsd-new-project
   - Plans will be better informed

6. **Track your savings over time**
   - Keep the cost breakdowns from each project
   - Look for patterns
   - Identify where research saves most

---

## Expected Costs

**Per project research phase:**
- Single round: ~500 tokens (~$0.015)
- Three rounds: ~1,500 tokens (~$0.045)
- Five rounds: ~2,500 tokens (~$0.075)

**Equivalent direct Claude:**
- Single round: ~3,500 tokens (~$0.105)
- Three rounds: ~10,000 tokens (~$0.30)
- Five rounds: ~15,000 tokens (~$0.45)

**Your savings:**
- Single round: 85%
- Three rounds: 85%
- Five rounds: 83%

In all cases, Ollama saves you ~85% on research costs.
