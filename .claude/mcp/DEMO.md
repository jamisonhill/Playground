# End-to-End Example: Building a Feature with Ollama Research Agent

## Scenario
You want to build a real-time dashboard component. You want:
1. Ollama to research best practices and generate architecture
2. Claude Code to implement the actual component

## Step 1: Pull Both Models

In terminal(s), pull both models (this unlocks the full dual-model strategy):

```bash
# Terminal 1: Fast model (pulls in ~30 seconds)
ollama pull mistral

# Terminal 2 (or wait for terminal 1): Deep model (pulls in ~5-10 minutes)
ollama pull deepseek-r1:70b

# Verify both are available
ollama list
```

Both models will run on your M3 Max. The server automatically picks the right one for each request.

## Step 2: Use Claude Code to Call the Research Agent

Once the model is pulled, open Claude Code in this directory and try:

```
I need to architect a real-time data dashboard for a React app. 
Research best practices for state management, real-time updates, 
and performance optimization. Then help me build it.
```

Claude Code will:
1. Detect the research need
2. Call the Ollama agent's `analyze` tool
3. Get structured insights from the local model
4. Use those insights to guide your implementation

## Step 3: Get an Implementation Plan

Ask Claude Code:

```
Based on your research, create a detailed plan for building a 
WebSocket-based dashboard component with React hooks. Include:
- Component structure
- State management approach
- Real-time update strategy
- Testing considerations
```

Claude Code will:
1. Call the Ollama agent's `generate_plan` tool
2. Receive a multi-phase plan
3. Use it to guide actual implementation

## Step 4: Review Architecture Before Implementing

Ask:

```
I'm thinking of using Context API for state management. Review 
this approach for a dashboard with 50+ real-time metrics updating 
every second. Should I use Redux instead? What are the tradeoffs?
```

Claude Code will:
1. Call the Ollama agent's `review_architecture` tool
2. Get detailed pros/cons and recommendations
3. Help you decide before writing code

## Step 5: Implement with Confidence

Now Claude Code can write the actual component:

```
Based on the plan and architecture review, implement the Dashboard 
component with:
- WebSocket connection management
- Real-time data updates
- Performance optimizations (memoization, virtualization)
- Error handling
```

Claude Code will:
- Write the actual code
- Handle file operations
- Run tests
- Make commits

---

## Dual-Model Strategy in Action

The server automatically selects the right model for speed vs. depth:

### Fast Iteration (Mistral)

When you ask quick questions, you get **2-3 second responses**:

```
You: "Should we use Redux or Zustand?"

Ollama (mistral): Returns quick comparison in ~2 seconds
[mistral (fast)]
Both are viable. Key difference: Redux is more mature/enterprise,
Zustand is lighter/faster. For a small team, use Zustand.
```

### Deep Analysis (Deepseek)

When you ask complex questions, you get **15-30 second responses** with thorough reasoning:

```
You: "Analyze our state management architecture for a real-time 
dashboard with 100+ metrics updating every second. We currently 
use Context API but it's causing performance issues. What are the 
architectural implications of switching to Zustand vs Redux vs 
native useReducer with custom hooks? Consider performance, 
maintainability, developer experience, and testing."

Ollama (deepseek-r1:70b): Returns 1500+ word analysis in ~20 seconds
[deepseek-r1:70b (deep)]
PERFORMANCE ANALYSIS:
Context API: O(n) re-renders of all consumers...
Zustand: O(1) subscriptions, fine-grained...
Redux: Mature optimization patterns...

RECOMMENDATION MATRIX:
For your 100+ metrics case, Zustand + Web Workers is optimal because...
```

### Auto-Selection Example

You **don't need to think about model selection**. The server is smart:

```
Short question:
  You: "Is Tailwind good for this?"
  → Uses mistral (fast) — 2 seconds

Long question:
  You: "Analyze these 800 characters of requirements and code..."
  → Auto-upgrades to deepseek (deep) — 20 seconds

Planning request:
  You: "Generate a plan for..."
  → Always uses deepseek (deep) — thorough planning

Brainstorming request:
  You: "Brainstorm solutions for..."
  → Uses mistral (fast) — rapid ideation
```

---

## Real Examples of Tool Usage

### Example 1: Analyzing a Tricky Problem

**You ask:**
```
Analyze this problem: Our React app is slowing down when 
updating 100+ metrics in real-time. We're using Context API 
and it's re-rendering the whole dashboard tree. What are the 
architectural options?
```

**Ollama returns (via analyze tool):**
```
Analysis of Real-Time Dashboard Optimization:

1. ROOT CAUSE: Context API limitations
   - Context changes trigger all consumers to re-render
   - No built-in optimization for partial updates
   - Unsuitable for high-frequency updates

2. ARCHITECTURAL OPTIONS:

   A) Zustand (Recommended for this case)
      Pros: Minimal re-renders, selective subscriptions, <2KB
      Cons: Learning curve, less ecosystem than Redux
      Best for: High-frequency updates, performance-critical
      
   B) Jotai/Recoil
      Pros: Atomic state, fine-grained reactivity
      Cons: More complex API
      Best for: Complex state dependencies
      
   C) Redux + React Query
      Pros: Mature, great DevTools, large ecosystem
      Cons: Boilerplate, heavier bundle
      Best for: Large teams, enterprise apps

3. DATA LAYER PATTERNS:
   - Use Web Workers for update processing
   - Debounce/throttle high-frequency updates
   - Batch updates into micro-transactions
   - Virtual scrolling for large tables

4. IMPLEMENTATION ROADMAP:
   - Phase 1: Migrate to Zustand (estimated: 2-3 days)
   - Phase 2: Add Web Worker for data processing (1 day)
   - Phase 3: Implement virtual scrolling (1 day)
```

**Claude Code then:**
- Uses this analysis to create a detailed plan
- Implements Zustand setup, Web Workers, and optimization
- Tests performance improvements

### Example 2: Planning a Complex Feature

**You ask:**
```
Generate a plan for implementing OAuth2 integration with Google 
and Microsoft in our existing React SPA. We use Express backend 
and PostgreSQL. Our constraint: zero downtime during rollout.
```

**Ollama returns (via generate_plan):**
```
OAuth2 Integration Implementation Plan

PHASE 1: Backend Setup (Day 1-2)
  Task 1.1: Install passport-google-oauth20, passport-microsoft
  Task 1.2: Create oauth_providers table (stores refresh tokens)
  Task 1.3: Implement /auth/google/callback endpoint
  Task 1.4: Implement /auth/microsoft/callback endpoint
  Task 1.5: Add token refresh job (background task)
  
PHASE 2: Frontend Integration (Day 2-3)
  Task 2.1: Create OAuth button components
  Task 2.2: Implement login flow with redirect
  Task 2.3: Handle token storage (localStorage vs sessionStorage)
  Task 2.4: Add logout functionality
  Task 2.5: Create protected route wrapper
  
PHASE 3: Safety & Rollout (Day 3-4)
  Task 3.1: Feature flag for new auth (disabled by default)
  Task 3.2: Gradual rollout (10% → 50% → 100% users)
  Task 3.3: Fallback to old auth if issues detected
  Task 3.4: Monitoring and alerts for failures
  Task 3.5: Database backup before rollout
  
DEPENDENCIES:
  - Backend tasks must complete before frontend
  - Database migration tested on staging first
  - Feature flag infrastructure required

SUCCESS CRITERIA:
  - Zero auth failures during rollout
  - Sub-100ms OAuth redirect time
  - 95% of users successfully migrated in 1 week
```

**Claude Code then:**
- Breaks this into actionable Git commits
- Implements each phase
- Handles database migrations
- Sets up feature flags

---

## How the MCP Bridge Works

```
Claude Code: "Analyze this architecture problem"
    ↓
Claude Code calls: tools.analyze({query: "...", context: "..."})
    ↓
MCP Server crafts optimized prompt:
  "You are a senior software architect. Analyze this question 
   thoroughly and provide actionable insights..."
    ↓
Ollama (local LLM) processes prompt
    ↓
Ollama returns structured response
    ↓
MCP Server returns response to Claude Code
    ↓
Claude Code uses analysis to inform implementation
```

The key advantage: Ollama works on YOUR machine, keeps research private,
and stays focused on architecture/planning (where local LLMs excel).

---

## Performance Notes

**On M3 Max with 64GB unified memory:**

| Model | Response Time | RAM Used | Best For |
|-------|---------------|----------|----------|
| **Mistral** | ~2-3 seconds | ~5GB | Quick feedback, ideation |
| **Deepseek-R1** | ~10-30 seconds | ~35GB | Planning, architecture, deep reasoning |

**Important notes:**
- First response of each session is slightly slower (loading to memory)
- Subsequent responses are faster once model is loaded
- Both models coexist — server loads only what's needed
- 64GB unified memory easily handles both

**Typical workflow:**
1. Ask a quick question → mistral answers in 2 seconds
2. Ask for planning → deepseek answers in 20 seconds
3. Continue iterating — both models already loaded and fast
4. No model switching cost after first use

**Getting the most from dual models:**
- Use mistral for rapid brainstorming/iteration
- Use deepseek for important decisions/planning
- Let auto-selection handle the rest
- Server is optimized for your workflow
