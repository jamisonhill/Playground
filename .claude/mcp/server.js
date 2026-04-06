#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  TextContent,
  Tool,
} from '@modelcontextprotocol/sdk/types.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

const OLLAMA_API = process.env.OLLAMA_API || 'http://localhost:11434';

// Model selection strategy
const MODELS = {
  fast: 'mistral',      // Quick analysis, brainstorming
  deep: 'deepseek-r1:70b', // Complex planning, architecture review
};

// Auto-select based on request complexity
function selectModel(toolName, inputSize) {
  // Use complexity heuristic: if input is large or task is complex, use deepseek
  const isComplexTask = ['generate_plan', 'review_architecture'].includes(toolName);
  const isLargeInput = (inputSize || '').length > 500;

  return (isComplexTask || isLargeInput) ? MODELS.deep : MODELS.fast;
}

const server = new Server({
  name: 'ollama-mcp',
  version: '1.0.0',
});

// Define available tools
const tools = [
  {
    name: 'analyze',
    description: 'Quick analysis of a problem (uses fast model by default). For research, quick feedback, and simple questions. Automatically upgrades to deep model for complex inputs.',
    inputSchema: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'The research or analysis question',
        },
        context: {
          type: 'string',
          description: 'Optional context (code snippets, requirements, constraints)',
        },
        model: {
          type: 'string',
          description: 'Optional: "fast" (mistral) or "deep" (deepseek-r1:70b). Auto-selected based on complexity.',
          enum: ['fast', 'deep'],
        },
      },
      required: ['query'],
    },
  },
  {
    name: 'generate_plan',
    description: 'Generate a structured implementation plan using deep model for thorough analysis.',
    inputSchema: {
      type: 'object',
      properties: {
        task: {
          type: 'string',
          description: 'The task or feature to plan',
        },
        constraints: {
          type: 'string',
          description: 'Technical constraints, deadlines, or requirements',
        },
        context: {
          type: 'string',
          description: 'Project context or existing codebase structure',
        },
        model: {
          type: 'string',
          description: 'Optional: "fast" (mistral) or "deep" (deepseek-r1:70b). Defaults to deep for planning.',
          enum: ['fast', 'deep'],
        },
      },
      required: ['task'],
    },
  },
  {
    name: 'review_architecture',
    description: 'Deep review of architectural decisions using deep model for thorough analysis.',
    inputSchema: {
      type: 'object',
      properties: {
        proposal: {
          type: 'string',
          description: 'The architecture or design proposal to review',
        },
        criteria: {
          type: 'string',
          description: 'Evaluation criteria (performance, maintainability, scalability, etc.)',
        },
        model: {
          type: 'string',
          description: 'Optional: "fast" (mistral) or "deep" (deepseek-r1:70b). Defaults to deep for reviews.',
          enum: ['fast', 'deep'],
        },
      },
      required: ['proposal'],
    },
  },
  {
    name: 'brainstorm',
    description: 'Quick brainstorming using fast model for creative ideation.',
    inputSchema: {
      type: 'object',
      properties: {
        problem: {
          type: 'string',
          description: 'The problem to brainstorm solutions for',
        },
        constraints: {
          type: 'string',
          description: 'Any constraints or limitations',
        },
        model: {
          type: 'string',
          description: 'Optional: "fast" (mistral) or "deep" (deepseek-r1:70b). Defaults to fast for quick brainstorming.',
          enum: ['fast', 'deep'],
        },
      },
      required: ['problem'],
    },
  },
];

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools,
}));

// Query Ollama API
async function queryOllama(model, prompt) {
  try {
    const response = await fetch(`${OLLAMA_API}/api/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model,
        prompt,
        stream: false,
        temperature: 0.7,
      }),
    });

    if (!response.ok) {
      return {
        error: `Ollama API error: ${response.status}`,
        details: await response.text(),
      };
    }

    const data = await response.json();
    return { response: data.response };
  } catch (error) {
    return { error: `Failed to reach Ollama: ${error.message}` };
  }
}

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  let prompt = '';
  let modelKey = args.model; // User-specified model, if any

  switch (name) {
    case 'analyze':
      // Auto-select if not specified: fast for short queries, deep for long/complex
      if (!modelKey) {
        const querySize = (args.query || '').length + (args.context || '').length;
        modelKey = querySize > 500 ? 'deep' : 'fast';
      }
      prompt = `You are a senior software architect and research analyst. Analyze this question thoroughly and provide actionable insights.\n\nQuestion: ${args.query}\n${args.context ? `\nContext:\n${args.context}` : ''}\n\nProvide a structured analysis with key insights and recommendations.`;
      break;

    case 'generate_plan':
      // Defaults to deep for comprehensive planning
      if (!modelKey) modelKey = 'deep';
      prompt = `You are an expert software architect. Create a detailed implementation plan for this task.\n\nTask: ${args.task}\n${args.constraints ? `\nConstraints: ${args.constraints}` : ''}\n${args.context ? `\nContext:\n${args.context}` : ''}\n\nProvide a step-by-step plan with clear phases, dependencies, and success criteria.`;
      break;

    case 'review_architecture':
      // Defaults to deep for thorough review
      if (!modelKey) modelKey = 'deep';
      prompt = `You are an experienced architect. Review this architectural proposal and provide detailed feedback.\n\nProposal:\n${args.proposal}\n${args.criteria ? `\nEvaluation Criteria: ${args.criteria}` : ''}\n\nProvide strengths, weaknesses, potential issues, and improvements.`;
      break;

    case 'brainstorm':
      // Defaults to fast for quick ideation
      if (!modelKey) modelKey = 'fast';
      prompt = `You are a creative problem solver. Brainstorm multiple approaches to this problem.\n\nProblem: ${args.problem}\n${args.constraints ? `\nConstraints: ${args.constraints}` : ''}\n\nProvide at least 3-5 distinct approaches with pros/cons for each.`;
      break;

    default:
      return {
        content: [
          {
            type: 'text',
            text: `Unknown tool: ${name}`,
          },
        ],
        isError: true,
      };
  }

  // Resolve model key to actual model name
  const selectedModel = MODELS[modelKey] || MODELS.fast;
  const modelDisplay = modelKey === 'deep' ? 'deepseek-r1:70b (deep)' : 'mistral (fast)';

  const result = await queryOllama(selectedModel, prompt);

  if (result.error) {
    return {
      content: [
        {
          type: 'text',
          text: `[${modelDisplay}]\n\nError: ${result.error}\n${result.details || ''}`,
        },
      ],
      isError: true,
    };
  }

  return {
    content: [
      {
        type: 'text',
        text: `[${modelDisplay}]\n\n${result.response}`,
      },
    ],
  };
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('Ollama MCP Server running on stdio');
}

main().catch(console.error);
