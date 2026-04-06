# Dual-Model Strategy: Mistral + Deepseek-R1

## Overview

Your MCP server now uses **two models strategically**:

- **Mistral** (7B) — Fast, lightweight, great for quick analysis and brainstorming
- **Deepseek-R1** (70B) — Powerful, thorough, ideal for complex planning and architecture review

The server **automatically selects** the right model based on your request, but you can also **explicitly choose** which one to use.

## Default Strategy

| Tool | Default Model | Reasoning |
|------|---------------|-----------|
| `analyze` | Auto-select* | Quick queries use mistral, complex ones use deepseek |
| `generate_plan` | deepseek-r1 | Planning needs thorough reasoning |
| `review_architecture` | deepseek-r1 | Architecture reviews need deep analysis |
| `brainstorm` | mistral | Ideation is faster with mistral |

*Auto-select logic: If query + context > 500 chars, uses deepseek; otherwise mistral.

## Explicit Model Selection

Each tool accepts an optional `model` parameter. When Claude Code calls tools, it can specify:

```json
{
  "tool": "analyze",
  "args": {
    "query": "Your question here",
    "context": "Optional context",
    "model": "fast"  // or "deep"
  }
}
```

## Examples

### Auto-Selection (Default)

**Short, quick question:**
```
analyze: {
  query: "What's a good caching strategy?"
}
// → Uses mistral (fast)
```

**Long, complex question:**
```
analyze: {
  query: "Analyze our current React state management with Context API...[500+ chars]",
  context: "[code snippets and architectural details]"
}
// → Uses deepseek (auto-upgraded due to size)
```

### Explicit Selection

**Force fast model for quick feedback:**
```
analyze: {
  query: "Should we use TypeScript or JavaScript?",
  model: "fast"
  // → Always uses mistral, ~2 seconds
}
```

**Force deep model for thorough analysis:**
```
brainstorm: {
  problem: "Generate 10 ways to optimize database queries",
  model: "deep"
  // → Uses deepseek for more creative solutions, ~15 seconds
}
```

## Response Headers

Each response is tagged with which model was used:

```
[mistral (fast)]
Your response here...

[deepseek-r1:70b (deep)]
Your response here...
```

This helps you understand performance/quality tradeoffs.

## Performance on M3 Max (64GB)

| Model | Typical Response Time | Quality | Use Case |
|-------|----------------------|---------|----------|
| Mistral | 2-3 seconds | Good | Quick feedback, brainstorming, simple analysis |
| Deepseek-R1 | 10-30 seconds | Excellent | Planning, architecture, complex reasoning |

First response is slower (loading model into memory). Subsequent responses are faster.

## When to Use Each

### Use Mistral When:
- ✅ You need quick feedback (< 5 seconds)
- ✅ The question is straightforward
- ✅ You're brainstorming multiple ideas quickly
- ✅ You're exploring options and want rapid iteration

**Examples:**
- "Quick design review of this component"
- "Suggest names for this function"
- "5 ways to implement this feature"
- "What library should we use?"

### Use Deepseek When:
- ✅ You need deep reasoning (10-30 seconds acceptable)
- ✅ The question is complex or multi-faceted
- ✅ You're planning a major feature
- ✅ You need thorough pros/cons analysis
- ✅ You're reviewing critical architectural decisions

**Examples:**
- "Create a 3-phase implementation plan for OAuth2"
- "Review this microservices architecture"
- "Analyze state management patterns for high-frequency updates"
- "Plan a database migration strategy"

## Cost & Resource Implications

Both models run **locally on your M3 Max** (no API calls, no per-request costs):

- **Mistral**: ~5GB RAM, uses 2-3 cores
- **Deepseek**: ~35GB RAM, uses 4-6 cores (M3 Max easily handles this)

Since you have 64GB unified memory, you can comfortably run either model. The server loads only what's needed.

## How Auto-Selection Works

When you don't specify a model, the server uses this logic:

```javascript
if (tool === 'generate_plan' || tool === 'review_architecture') {
  use deepseek  // Planning/review always needs depth
} else if (tool === 'brainstorm') {
  use mistral   // Ideation is faster with mistral
} else if (tool === 'analyze') {
  // Input size heuristic:
  if (query + context > 500 characters) {
    use deepseek  // Complex analysis
  } else {
    use mistral   // Quick analysis
  }
}
```

This means **you usually don't need to think about model selection** — it just works.

## Claude Code Integration

When Claude Code calls the Ollama tools:

1. **Claude Code identifies the task** (research vs. planning vs. review)
2. **MCP Server auto-selects the appropriate model**
3. **Local model processes the request**
4. **Response is tagged with model used** so you know the tradeoff

Example flow:
```
Claude Code: "Generate a plan for OAuth2 implementation"
  → detect: generate_plan task
  → auto-select: deepseek-r1
  → wait 15-20 seconds for deep analysis
  → return tagged response with [deepseek-r1:70b (deep)]

Claude Code: "Brainstorm ways to optimize CSS"
  → detect: brainstorm task
  → auto-select: mistral
  → wait 2-3 seconds for ideas
  → return tagged response with [mistral (fast)]
```

## Testing Both Models

Run the test suite to verify both models work:

```bash
cd /Users/jamisonhill/Ai/Playground/.claude/mcp
npm test
```

Output will show:
```
2. Model availability:
   ✅ Both models ready
     - mistral (fast analysis)
     - deepseek-r1:70b (deep planning)

3. Testing inference...
   Testing mistral (fast)...
     ✅ Working
   Testing deepseek-r1:70b (deep)...
     ✅ Working
```

## Pulling Both Models

If you haven't pulled them yet:

```bash
# In separate terminals or sequentially:

# Fast model (pulls in ~30 seconds)
ollama pull mistral

# Deep model (pulls in ~5-10 minutes depending on connection)
ollama pull deepseek-r1:70b

# Check what's available:
ollama list
```

Once both are pulled, the MCP server automatically handles model selection.

## Summary

You now have a **cost-free, on-machine research agent** that:

- ✅ Automatically picks the right model for the task
- ✅ Gives you fast feedback when you need it (mistral)
- ✅ Provides deep analysis when you need depth (deepseek)
- ✅ Lets you override the default if you want specific behavior
- ✅ Keeps all research private (never leaves your machine)
- ✅ Integrates seamlessly with Claude Code's coding workflow

The dual-model approach is the sweet spot for your workflow: **fast iteration + deep thinking**, all local.
