# Ollama MCP Server Usage Guide

## Setup

1. **Start Ollama** (if not already running):
   ```bash
   ollama serve
   ```

2. **Pull a model** in a new terminal:
   ```bash
   # Recommended: Deep analysis and planning
   ollama pull deepseek-r1:70b
   
   # Or faster alternatives:
   ollama pull mistral
   ollama pull neural-chat
   ollama pull llama2
   ```

3. **Server is auto-loaded** via `.mcp.json` when you start Claude Code

## Available Tools

When Claude Code starts, it automatically loads the Ollama MCP server with 4 research-focused tools. **Each tool intelligently selects between mistral (fast) and deepseek (deep) based on your request.**

### 1. **analyze**
Quick or deep analysis depending on complexity. Auto-upgrades for complex queries.
```
analyze: {
  query: "Architecture question or research topic",
  context: "(optional) relevant code, constraints, or background",
  model: "(optional) 'fast' or 'deep' to override auto-selection"
}
```

**When to use:**
- Quick feedback on decisions
- Technology research
- Requirements analysis
- Problem decomposition

**Model behavior:**
- Short queries → mistral (fast, ~2 seconds)
- Long/complex queries → deepseek (deep, ~15 seconds)
- You can override with `model: "fast"` or `model: "deep"`

### 2. **generate_plan**
Structured implementation planning (defaults to deepseek for thoroughness)
```
generate_plan: {
  task: "What you want to build",
  constraints: "(optional) timeline, tech limits, requirements",
  context: "(optional) existing codebase structure",
  model: "(optional) 'fast' for quick outline or 'deep' for comprehensive plan"
}
```

**When to use:**
- Multi-phase planning
- Feature implementation strategy
- Refactoring strategy
- Complex project breakdown

**Model behavior:**
- Defaults to deepseek for thorough plans
- Use `model: "fast"` for quick outlines (~3 seconds)

### 3. **review_architecture**
Detailed architecture review (defaults to deepseek for thorough analysis)
```
review_architecture: {
  proposal: "Architecture or design description",
  criteria: "(optional) evaluation criteria (performance, maintainability, etc.)",
  model: "(optional) 'fast' for quick feedback or 'deep' for thorough review"
}
```

**When to use:**
- Pre-implementation reviews
- Scalability evaluation
- Tech stack validation
- Design quality assessment

**Model behavior:**
- Defaults to deepseek for comprehensive reviews
- Use `model: "fast"` for quick gut-check (~2 seconds)

### 4. **brainstorm**
Creative ideation (defaults to mistral for fast iteration)
```
brainstorm: {
  problem: "Problem to solve",
  constraints: "(optional) limitations",
  model: "(optional) 'fast' for quick ideas or 'deep' for thorough exploration"
}
```

**When to use:**
- Multiple solution exploration
- Creative problem-solving
- Design alternative generation
- Feature ideation

**Model behavior:**
- Defaults to mistral for rapid ideation (~2 seconds)
- Use `model: "deep"` for more sophisticated solutions (~15 seconds)

## Workflow Example

```
User: "I need to plan a React component library. Can you research best practices?"

Claude Code: I'll consult the Ollama research agent for this.
(uses analyze tool with ollama-local)

Ollama Agent: Returns structured analysis of component library patterns,
design systems, testing strategies, and documentation approaches.

Claude Code: Based on the research, I'll now create a detailed plan
and help with the actual implementation.
```

## Model Selection

The server automatically chooses the right model for each task:

**Auto-selection rules:**
- **analyze**: Fast for short queries, deep for complex ones (>500 chars)
- **generate_plan**: Deep (for comprehensive planning)
- **review_architecture**: Deep (for thorough review)
- **brainstorm**: Fast (for rapid ideation)

**Override on demand:**
Add `model: "fast"` or `model: "deep"` to any tool to override auto-selection.

**Performance on M3 Max:**
- Mistral (fast): ~2-3 seconds per response
- Deepseek-R1 (deep): ~10-30 seconds per response

See `MODEL_STRATEGY.md` for detailed comparison and use cases.

## Troubleshooting

**Q: MCP server not connecting**
- Ensure Ollama is running: `lsof -i :11434`
- Check logs: `ollama logs` (or check activity monitor)
- Verify model is pulled: `ollama list`

**Q: Slow responses**
- Deepseek-r1:70b is large — use `mistral` or `neural-chat` for faster testing
- On M3 Max with 64GB RAM, deepseek should complete in seconds

**Q: How do I use this from Claude Code?**
- Just ask Claude Code questions that need research:
  - "Research best practices for X"
  - "Create a plan for building Y"
  - "Review this architecture approach"
- Claude Code will automatically use the Ollama agent when appropriate

## Architecture

```
Claude Code
    ↓
.mcp.json (MCP config)
    ↓
server.js (Node.js MCP server)
    ↓
Ollama API (http://localhost:11434)
    ↓
Local Model (deepseek-r1:70b, etc.)
```

The MCP server acts as a bridge that translates structured tool calls into optimized prompts for your local Ollama model, then returns actionable responses to Claude Code.
