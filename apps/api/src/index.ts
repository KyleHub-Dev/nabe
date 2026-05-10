import Fastify from 'fastify';

const app = Fastify({ logger: true });

const placeholderRoutes = [
  'users',
  'devices',
  'dns-instances',
  'adguard-engine',
  'querylog',
  'speiche/enrollment',
  'speiche/heartbeat'
];

app.get('/health', async () => ({
  status: 'ok',
  service: 'nabe-api'
}));

for (const route of placeholderRoutes) {
  app.get(`/v1/${route}`, async () => ({
    route,
    status: 'placeholder'
  }));
}

const host = process.env.NABE_API_HOST ?? '0.0.0.0';
const port = Number(process.env.NABE_API_PORT ?? 8080);

await app.listen({ host, port });
