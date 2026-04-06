# Quick Start: Dual-Model Ollama Research Agent

## 1-Minute Setup

**Pull both models** (do once):
```bash
ollama pull mistral           # ~30 seconds
ollama pull deepseek-r1:70b   # ~5-10 minutes
```

**Start Claude Code:**
```bash
cd /Users/jamisonhill/Ai/Playground
claude
```

**Done!** The MCP server is auto-loaded and ready.

---

## How to Use

Just ask Claude Code questions that need research:

### Quick Questions (Mistral - 2 seconds)
```
"Quick design review: use Redux or Zustand?"
"What's a good name for this component?"
"5 ways to improve database query performance"
```
→ Gets mistral response automatically

### Complex Questions (Deepseek - 20 seconds)
```
"Analyze our state management for a dashboard with 100+ 
real-time metrics updating every second. Context API is 
slow. What architecture should we use?"
```
→ Auto-upgrades to deepseek for thorough analysis

### Planning (Deepseek - 20 seconds)
```
"Create a 3-phase plan for implementing OAuth2 with 
Google and Microsoft, zero-downtime rollout."
```
→ Always uses deepseek

### Brainstorming (Mistral - 2 seconds)
```
"Brainstorm 5 creative ways to handle real-time notifications"
```
→ Always uses mistral

---

## Response Format

Each response is tagged with the model used:

```
[mistral (fast)]
Quick answer about X...

[deepseek-r1:70b (deep)]
Thorough analysis of X...
```

This helps you see the speed/quality tradeoff.

---

## Manual Model Selection (Optional)

If you want to force a specific model, you can ask Claude Code:

```
"Analyze this quickly: [question]" 
→ Tells Claude Code to use mistral

"Do a deep analysis of [complex question]"
→ Tells Claude Code to use deepseek
```

Or directly specify in the tool call:
```json
{
  "tool": "analyze",
  "args": {
    "query": "Your question",
    "model": "fast"  // or "deep"
  }
}
```

---

## Performance Expectations

| Scenario | Time | Why |
|----------|------|-----|
| Quick brainstorm | ~2 sec | Mistral is fast |
| Simple analysis | ~2-3 sec | Mistral handles it |
| Complex planning | ~20 sec | Deepseek thinks through it |
| Multiple requests | ~2 sec each | Models stay loaded |

**First response might be ~5 seconds slower** as models load into memory. After that, it's fast.

---

## What Gets What?

**Mistral (Fast Model)**
- analyze (short queries)
- brainstorm
- Any quick feedback requests

**Deepseek (Deep Model)**
- analyze (long/complex queries)
- generate_plan
- review_architecture
- Any planning/architecture request

**Auto-selection rule:** Server picks based on task type and query size. You don't have to think about it.

---

## Common Patterns

### Pattern 1: Rapid Iteration
```
1. "Quick idea: use Postgres or SQLite?" (mistral, 2 sec)
2. "Brainstorm schema design" (mistral, 2 sec)
3. "Review this schema" (deepseek, 20 sec)
4. "Plan migration strategy" (deepseek, 20 sec)
```

### Pattern 2: Deep Dive
```
1. "Analyze our auth system for vulnerabilities" 
   (deepseek, 20 sec)
2. "Plan OAuth2 implementation" 
   (deepseek, 20 sec)
3. "Review proposed changes" 
   (deepseek, 20 sec)
```

### Pattern 3: Get Code Written
```
1. "Research best practices for X" (mistral, 2 sec)
2. "Create plan for implementing X" (deepseek, 20 sec)
3. Claude Code writes the actual code
```

---

## Workflow: Research → Plan → Code

```
You ask Ollama (research): "What's best for real-time updates?"
  ↓ (mistral: 2 seconds)
Ollama answers: "WebSockets are better than polling because..."

You ask Claude Code: "Create a plan for WebSocket implementation"
  ↓ (deepseek: 20 seconds)
Ollama/Claude Code create: 3-phase plan with clear steps

You ask Claude Code: "Now implement it"
  ↓
Claude Code writes actual React component code, files, tests
```

**You keep the benefits of both models:**
- ✅ Fast research/ideation with mistral
- ✅ Deep analysis with deepseek
- ✅ Claude Code handles actual coding

---

## Troubleshooting

**Q: Model is slow**
- First response is slower while loading. Subsequent ones are fast.
- Deepseek (20 sec) is slower than mistral (2 sec) — that's normal.

**Q: Only one model is installed**
- Install both: `ollama pull mistral` and `ollama pull deepseek-r1:70b`

**Q: Want to check what's installed**
```bash
ollama list
```

**Q: Server not connecting**
```bash
# Check Ollama is running
lsof -i :11434

# Test it
cd /Users/jamisonhill/Ai/Playground/.claude/mcp
npm test
```

---

## Files for Reference

- `USAGE.md` — Full documentation of all tools
- `MODEL_STRATEGY.md` — Deep dive on dual-model approach
- `DEMO.md` — Real-world examples and use cases
- `test.js` — Test script (run: `npm test`)

---

## Summary

You now have a **cost-free, local research agent** that:

✅ Automatically picks the right model (fast vs. deep)
✅ Gives you 2-second feedback when you need it (mistral)
✅ Provides 20-second deep analysis when needed (deepseek)
✅ Works seamlessly with Claude Code's coding workflow
✅ Keeps all research private (never leaves your M3 Max)

Just ask Claude Code questions like you normally would. The Ollama agent handles the research part automatically.

**Start with:** Pull both models, then use Claude Code as normal!
