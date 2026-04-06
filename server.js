import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = express();
const PORT = 5000;

// Middleware
app.use(express.json());
app.use(express.static('public'));

// Routes
app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, 'public', 'index.html'));
});

app.get('/api/recipes', (req, res) => {
  // TODO: Load recipes from JSON
  res.json({ recipes: [] });
});

// Start server
app.listen(PORT, () => {
  console.log(`✓ Express server running on http://localhost:${PORT}`);
});
