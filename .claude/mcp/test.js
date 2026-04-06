#!/usr/bin/env node

const OLLAMA_API = process.env.OLLAMA_API || 'http://localhost:11434';

async function testOllama() {
  console.log(`Testing Ollama at ${OLLAMA_API}...\n`);

  try {
    // Test 1: Check server health
    console.log('1. Checking Ollama server...');
    const healthResponse = await fetch(`${OLLAMA_API}/api/tags`, {
      method: 'GET',
    });

    if (!healthResponse.ok) {
      console.error(`   ❌ Server not responding: ${healthResponse.status}`);
      return;
    }

    const models = await healthResponse.json();
    console.log(`   ✅ Server is running`);
    console.log(`   Available models:`, models.models?.length || 0);
    if (models.models?.length > 0) {
      models.models.forEach((m) => {
        console.log(`     - ${m.name} (${(m.size / 1e9).toFixed(2)}GB)`);
      });
    } else {
      console.log(`     (no models pulled yet)`);
    }

    // Test 2: Check for required models
    console.log(`\n2. Model availability:`);
    const hasMistral = models.models?.some((m) => m.name.includes('mistral'));
    const hasDeepseek = models.models?.some((m) => m.name.includes('deepseek-r1'));

    if (hasMistral && hasDeepseek) {
      console.log(`   ✅ Both models ready`);
      console.log(`     - mistral (fast analysis)`);
      console.log(`     - deepseek-r1:70b (deep planning)`);
    } else {
      console.log(`   ⚠️  Missing models:`);
      if (!hasMistral) {
        console.log(`     To pull fast model: ollama pull mistral`);
      }
      if (!hasDeepseek) {
        console.log(`     To pull deep model: ollama pull deepseek-r1:70b`);
      }
    }

    // Test 3: Try inference with available models
    if (models.models?.length > 0) {
      console.log(`\n3. Testing inference...`);

      // Test with mistral if available
      if (hasMistral) {
        console.log(`   Testing mistral (fast)...`);
        const fastResponse = await fetch(`${OLLAMA_API}/api/generate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            model: 'mistral',
            prompt: 'Respond with exactly one sentence about machine learning.',
            stream: false,
            temperature: 0.5,
          }),
        });

        if (fastResponse.ok) {
          const result = await fastResponse.json();
          console.log(`     ✅ Working`);
          console.log(`     Response: ${result.response.substring(0, 80)}...`);
        } else {
          console.log(`     ❌ Failed: ${fastResponse.status}`);
        }
      }

      // Test with deepseek if available
      if (hasDeepseek) {
        console.log(`   Testing deepseek-r1:70b (deep)...`);
        const deepResponse = await fetch(`${OLLAMA_API}/api/generate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            model: 'deepseek-r1:70b',
            prompt: 'Respond with exactly one sentence about system architecture.',
            stream: false,
            temperature: 0.5,
          }),
        });

        if (deepResponse.ok) {
          const result = await deepResponse.json();
          console.log(`     ✅ Working`);
          console.log(`     Response: ${result.response.substring(0, 80)}...`);
        } else {
          console.log(`     ❌ Failed: ${deepResponse.status}`);
        }
      }
    }

    console.log('\n✅ Ollama is ready for MCP integration');
  } catch (error) {
    console.error(`\n❌ Error: ${error.message}`);
    console.error(
      'Make sure Ollama is running: ollama serve'
    );
  }
}

testOllama();
