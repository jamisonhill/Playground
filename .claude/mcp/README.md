# Ollama MCP Server for Claude Code

A local, cost-free research agent that bridges Claude Code to your Ollama instance, using dual models for intelligent speed/depth tradeoffs.

## Status

✅ **Ready to use with Mistral**
⏳ **Waiting for Deepseek-R1 pull**

```
Ollama Server: Running (http://localhost:11434)
Models Installed: mistral:latest (4.37GB)
Models Needed: deepseek-r1:70b
MCP Server: Installed and configured
Test Result: ✅ Mistral inference working
```

## What You Have

- **MCP Server** (`server.js`): Bridges Claude Code → Ollama
- **Dual-Model Strategy**: Mistral (fast) + Deepseek (deep)
- **4 Research Tools**: analyze, generate_plan, review_architecture, brainstorm
- **Auto-Selection**: Smart model picking based on task complexity
- **Zero Cost**: Runs entirely on your M3 Max

## Getting Started

### Step 1: Pull Deepseek-R1 (5-10 minutes)

In a new terminal:
```bash
ollama pull deepseek-r1:70b
```

This downloads the 70B parameter model (~35GB). You already have mistral (4.37GB).

### Step 2: Start Claude Code

```bash
cd /Users/jamisonhill/Ai/Playground
claude
```

The MCP server is auto-loaded from `.mcp.json`.

### Step 3: Use It

Ask Claude Code research questions. Examples:

**Fast (Mistral, ~2 seconds):**
```
"Quick review: should we use Zustand or Redux?"
"5 brainstorm ideas for handling real-time updates"
```

**Deep (Deepseek, ~20 seconds):**
```
"Create a plan for implementing OAuth2 with zero-downtime rollout"
"Analyze our state management for performance issues"
"Review this microservices architecture"
```

The server automatically picks the right model.

---

## Architecture

```
Claude Code Session
        ↓
    .mcp.json (loads server)
        ↓
  server.js (MCP Bridge)
        ↓
    [Model Selector]
        ↓
  [Mistral 7B]  OR  [Deepseek-R1 70B]
        ↓
  http://localhost:11434
        ↓
  Your M3 Max (64GB RAM)
```

## Key Features

| Feature | Benefit |
|---------|---------|
| **Auto Model Selection** | Fast for quick questions, deep for planning |
| **Dual Models** | Mistral for speed, Deepseek for depth |
| **Zero Cost** | Runs locally, no API calls or subscriptions |
| **Private** | All research stays on your machine |
| **Seamless Integration** | Works with Claude Code's coding workflow |
| **Explicit Control** | Override model selection when needed |

## Available Tools

### 1. `analyze`
- **Default**: Auto-select (fast for short, deep for long queries)
- **When**: Research, quick feedback, problem analysis
- **Response**: Structured insights and recommendations

### 2. `generate_plan`
- **Default**: Deepseek (thorough planning)
- **When**: Multi-phase projects, implementation strategy
- **Response**: Step-by-step plan with phases and dependencies

### 3. `review_architecture`
- **Default**: Deepseek (comprehensive review)
- **When**: Design feedback, pre-implementation validation
- **Response**: Strengths, weaknesses, improvements

### 4. `brainstorm`
- **Default**: Mistral (rapid ideation)
- **When**: Multiple solution exploration, creative problem-solving
- **Response**: 3-5 distinct approaches with pros/cons

---

## Documentation

- **[QUICKSTART.md](QUICKSTART.md)** — 1-minute setup and examples (START HERE)
- **[USAGE.md](USAGE.md)** — Detailed tool documentation
- **[MODEL_STRATEGY.md](MODEL_STRATEGY.md)** — Deep dive on dual-model approach
- **[DEMO.md](DEMO.md)** — Real-world workflow examples

## Test Setup

Verify everything is working:
```bash
npm test
```

Expected output:
```
✅ Server is running
✅ Mistral is available
⏳ Waiting for deepseek-r1:70b...
✅ Mistral inference working
```

After pulling deepseek:
```
✅ Both models ready
✅ Mistral inference working
✅ Deepseek-R1 inference working
```

---

## Configuration

### MCP Server Config
File: `.mcp.json`
```json
{
  "mcpServers": {
    "ollama-local": {
      "command": "node",
      "args": ["/Users/jamisonhill/Ai/Playground/.claude/mcp/server.js"]
    }
  }
}
```

### Model Behavior
File: `server.js` (lines 14-18)
```javascript
const MODELS = {
  fast: 'mistral',           // 2-3 seconds
  deep: 'deepseek-r1:70b',  // 10-30 seconds
};
```

Change model names here if you use different versions.

---

## Typical Workflow

```
PHASE 1: Research (Mistral)
  You: "Quick overview of OAuth2"
  Agent: Returns overview in ~2 seconds

PHASE 2: Planning (Deepseek)
  You: "Create implementation plan"
  Agent: Returns detailed plan in ~20 seconds

PHASE 3: Implementation (Claude Code)
  You: "Now code it"
  Claude Code: Writes actual component, tests, etc.
```

All three phases work together seamlessly.

---

## Performance

**On M3 Max with 64GB unified memory:**

| Model | Speed | Quality | Use For |
|-------|-------|---------|---------|
| **Mistral** | 2-3 sec | Good | Quick feedback, brainstorming |
| **Deepseek-R1** | 15-30 sec | Excellent | Planning, architecture, deep reasoning |

**First response** is slower (model loads to memory). Subsequent responses are faster once loaded.

---

## What to Do Next

### Immediate (Next 5 minutes)

1. ✅ MCP server is installed
2. ✅ Mistral is ready
3. ⏳ **Pull deepseek-r1:70b**:
   ```bash
   ollama pull deepseek-r1:70b
   ```

### Then (Next 1 minute)

4. Start Claude Code:
   ```bash
   claude
   ```

### Then Use It

5. Ask research questions and Claude Code will consult the local agent

---

## Troubleshooting

**Q: MCP server not loading?**
```bash
# Verify Ollama is running
lsof -i :11434

# Test server
npm test
```

**Q: Only mistral works, no deepseek?**
- Pull it: `ollama pull deepseek-r1:70b`
- Wait for pull to complete
- Run `npm test` to verify

**Q: Response is slow?**
- First response: Model is loading (expected)
- Subsequent responses: Should be faster
- Deepseek is inherently slower than mistral (normal, worth it for planning)

**Q: Want to change default models?**
- Edit `server.js` lines 15-18
- Change `MODELS` object to point to different versions
- Restart Claude Code

---

## File Structure

```
.claude/mcp/
├── server.js           # MCP bridge to Ollama (auto-started)
├── package.json        # Dependencies (MCP SDK)
├── test.js             # Connection test script
├── README.md           # This file
├── QUICKSTART.md       # Quick start guide
├── USAGE.md            # Tool documentation
├── MODEL_STRATEGY.md   # Dual-model deep dive
└── DEMO.md             # Real-world examples

.mcp.json              # MCP server configuration (auto-loaded by Claude Code)
```

---

## Implementation Details

### How Auto-Selection Works

```javascript
if (task is "generate_plan" OR "review_architecture") {
  use deepseek  // Always thorough for planning/review
} else if (task is "brainstorm") {
  use mistral   // Always fast for ideation
} else if (task is "analyze") {
  if (query_length > 500 chars) {
    use deepseek  // Complex questions get deep analysis
  } else {
    use mistral   // Simple questions get fast answer
  }
}
```

You can override by specifying `model: "fast"` or `model: "deep"`.

### Response Format

Each response includes model used:
```
[mistral (fast)]
Your quick answer...

[deepseek-r1:70b (deep)]
Your comprehensive analysis...
```

This helps you see the speed/quality tradeoff.

---

## Advanced Usage

### Force Specific Model

Add `model` parameter to any tool:
```
analyze: {
  query: "Your question",
  model: "deep"  // Forces deepseek
}
```

### Custom Prompts

Edit prompts in `server.js` (lines 120-150). Each tool has a custom system prompt tuned for the task.

### Change Default Model

Update `MODELS` object in `server.js`:
```javascript
const MODELS = {
  fast: 'your-fast-model',
  deep: 'your-deep-model',
};
```

---

## Why This Setup?

1. **Cost**: Free after initial downloads (no API calls)
2. **Privacy**: All work stays on your machine
3. **Speed**: Mistral gives you instant feedback
4. **Quality**: Deepseek gives you thorough analysis
5. **Seamless**: Works with Claude Code's existing workflow

You get the best of both worlds: **fast iteration + deep thinking**.

---

## Next Steps

1. Pull deepseek: `ollama pull deepseek-r1:70b`
2. Start Claude Code: `claude`
3. Ask a research question
4. Watch the magic happen

See **[QUICKSTART.md](QUICKSTART.md)** for concrete examples!
