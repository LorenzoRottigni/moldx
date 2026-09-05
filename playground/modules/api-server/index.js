import express from 'express';
import cors from 'cors';

const app = express();
const port = process.env.PORT || 3000;

app.use(cors());
app.use(express.json());

const items = [];

app.get('/api/health', (_req, res) => {
  res.json({ status: 'ok', service: 'api-server', uptime: process.uptime() });
});

app.get('/api/items', (_req, res) => {
  res.json(items);
});

app.post('/api/items', (req, res) => {
  const item = { id: items.length + 1, ...req.body };
  items.push(item);
  res.status(201).json(item);
});

app.get('/api/items/:id', (req, res) => {
  const item = items.find((i) => i.id === Number(req.params.id));
  if (!item) return res.status(404).json({ error: 'not found' });
  res.json(item);
});

app.listen(port, () => {
  console.log(`[api-server] listening on :${port}`);
});
