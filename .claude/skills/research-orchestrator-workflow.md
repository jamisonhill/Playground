# Research Orchestrator - Execution Workflow

This file defines exactly how Claude should execute the research orchestrator skill.

## When User Calls: `/research-orchestrator "project scope"`

### Phase 1: Initial Prompt Generation

Claude should:

1. **Acknowledge the request**
   ```
   ✅ RESEARCH ORCHESTRATION STARTED
   PROJECT: [user's project scope]
   ```

2. **Generate research plan** (identify 2-3 key research questions)
   ```
   RESEARCH PLAN:
   1. [Key question 1]
   2. [Key question 2]
   3. [Key question 3]
   ```

3. **Generate first Ollama prompt** (specific, actionable, technical)
   ```
   STEP 1: Generate Ollama Research Prompt

   [COST: ~200 tokens]
   ← Claude generating optimized research prompt

   ---

   OLLAMA RESEARCH PROMPT:

   Copy and paste into terminal:
   ollama run mistral "[SPECIFIC, DETAILED RESEARCH PROMPT]"

   ---

   NEXT:
   Run the command, paste Ollama's response back here.
   Or say "continue research" to explore different angles.
   ```

4. **Cost indicator**
   ```
   [COST: ~200 tokens]
   ← This step consumed Claude tokens
   ```

### Phase 2: Process Ollama Response

When user pastes Ollama response, Claude should:

1. **Acknowledge receipt**
   ```
   [COST: ~300 tokens]
   ← Claude processing Ollama response + summarizing
   ```

2. **Extract key findings** in this format:
   ```
   ✅ RESEARCH SUMMARY - [TOPIC]

   KEY FINDINGS:
   1. [Finding 1 - concise, technical]
      - Supporting detail
      - Supporting detail
   
   2. [Finding 2]
      - Supporting detail
   
   CONFIDENCE: [High/Medium/Low] ([reason])
   ```

3. **Show accumulated research**
   ```
   RESEARCH SO FAR:
   Round 1: [Key finding 1]
   Round 2: [Key finding 2] (if multiple rounds)
   ```

4. **Show cost summary**
   ```
   COST SUMMARY SO FAR:
   - Research steps: FREE (Ollama)
   - Processing steps: ~[total] tokens
   - Total cost: ~[X] tokens
   - Equivalent without Ollama: ~[Y] tokens
   - Savings: ~[Z]%
   ```

5. **Offer next decision**
   ```
   NEXT DECISION:

   Option A: Continue research
   [FREE - Ollama will analyze next angle]
   "Continue research: [specific angle]"

   Option B: Move to GSD Planning
   [COST - Claude planning phase begins]
   "Move to GSD"

   Option C: Quick follow-up question
   [FREE + COST]
   "Quick follow-up: [question]"

   What would you like to do?
   ```

### Phase 3: Research Loop

If user says "Continue research: [angle]":

1. **Generate new Ollama prompt** targeting the new angle
   ```
   ✅ RESEARCH LOOP - Round [N]

   [COST: ~200 tokens]
   ← Claude generating refined research prompt

   ---

   OLLAMA RESEARCH PROMPT:

   Copy and paste:
   ollama run mistral "[NEW RESEARCH PROMPT]"
   ```

2. **When response comes back**: Repeat Phase 2 but integrate with previous findings
   ```
   ✅ RESEARCH SUMMARY - UPDATED

   KEY FINDINGS (CONSOLIDATED):
   1. [Integrated finding from both rounds]
   2. [New finding from round 2]
   3. [Synthesized recommendation]
   ```

3. **Keep accumulating** until user says "Move to GSD"

### Phase 4: GSD Integration

If user says "Move to GSD":

1. **Generate GSD context**
   ```
   ✅ READY FOR GSD PLANNING

   [COST: ~400 tokens]
   ← Claude generating GSD-ready context

   ---

   GSD CONTEXT (copy and use as context in /gsd-new-project):

   "PROJECT CONTEXT: [synthesized from all research]
   Key decisions: [from research]
   Architecture: [from research]
   Key risks: [from research]"
   ```

2. **Suggest GSD command**
   ```
   SUGGESTED GSD COMMAND:

   /gsd-new-project [Project Name]

   When prompted for context, paste the GSD CONTEXT above.
   ```

3. **Show complete cost breakdown**
   ```
   COMPLETE COST BREAKDOWN:

   Research Phase (Ollama): FREE
     - [N] research rounds

   Processing Phase (Claude): ~[total] tokens
     - Prompts: ~[X] tokens
     - Summaries: ~[Y] tokens
     - GSD context: ~[Z] tokens

   Without Ollama (hypothetical):
     - Would have cost: ~[X] tokens
     - You saved: ~[Y]%

   GSD Planning Phase (upcoming):
     - Will cost: ~[X]-[Y] tokens
     - Base cost (unavoidable)

   GSD Execution Phase (upcoming):
     - Will cost: ~[X]-[Y] tokens
     - Base cost (unavoidable)

   TOTAL PROJECT EFFICIENCY:
     - With Ollama: ~[X] tokens
     - Without Ollama: ~[Y] tokens
     - Project savings: ~[Z]%
   ```

4. **Next step**
   ```
   Next: Run /gsd-new-project as shown above.
   ```

## Cost Token Estimates

Use these estimates for cost display:

| Operation | Cost |
|-----------|------|
| Generate Ollama prompt | ~200 tokens |
| Summarize Ollama response | ~300 tokens |
| Integrate multi-round findings | ~350 tokens |
| Generate GSD context | ~400 tokens |
| Full research loop (3 rounds) | ~1,250 tokens |

Accuracy: ±10% depending on response length

## Key Rules

1. **Always mark costs**
   - Every Claude processing step: `[COST: ~X tokens]`
   - Every Ollama step: `[FREE]`
   - Summary at each decision point

2. **Always summarize Ollama responses**
   - Extract key findings
   - Integrate with previous research
   - Make it machine-readable for next steps

3. **Support unlimited research rounds**
   - Each round is FREE (Ollama)
   - Each summary is COST (Claude)
   - Build confidence before moving to GSD

4. **Make Ollama prompts specific**
   - Not: "Tell me about databases"
   - But: "Compare PostgreSQL vs TimescaleDB for time-series metrics at 10k ops/sec"

5. **Provide clear decision points**
   - Each phase ends with: continue / move on / quick question

6. **Track total savings**
   - Show what it would cost without Ollama
   - Show actual cost with Ollama
   - Display percentage savings

## Response Templates

### Cost Display
```
[FREE]
[COST: ~200 tokens]
[COST: ~300 tokens]
[COST SUMMARY]
[TOTAL PROJECT COST]
```

### Research Summary Format
```
KEY FINDINGS:
1. Finding with context
   - Detail
   - Detail

CONFIDENCE: High/Medium/Low
REASONING: Why we're confident
```

### Decision Point Format
```
NEXT DECISION:

Option A: Continue research
[FREE → COST]
"Command to continue"

Option B: Move to GSD
[COST]
"Command to proceed"

Option C: Quick follow-up
[FREE → COST]
"Type your question"
```

## Example Token Costs

**Small project (simple architecture):**
- Research 2 rounds: ~600 tokens
- Without Ollama: ~5000 tokens
- Savings: ~88%

**Medium project (moderate complexity):**
- Research 3-4 rounds: ~1200 tokens
- Without Ollama: ~8000 tokens
- Savings: ~85%

**Large project (complex system):**
- Research 5-6 rounds: ~2000 tokens
- Without Ollama: ~12000 tokens
- Savings: ~83%

---

## When This Workflow Is Active

Claude should follow this workflow whenever:
1. User calls `/research-orchestrator [project scope]`
2. User responds with Ollama output
3. User selects "continue research", "move to GSD", or "quick follow-up"

The workflow loops until user selects "Move to GSD", at which point it transitions to GSD planning.

---

## Integration with GSD

After research completes, the GSD context generated should:
- ✅ Be copy-paste ready for `/gsd-new-project`
- ✅ Include architectural decisions from research
- ✅ Include key risks and mitigations
- ✅ Include implementation approach
- ✅ Provide foundation for project scope

This ensures GSD planning starts with validated, researched context rather than assumptions.
