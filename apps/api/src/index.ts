import Fastify from 'fastify';
import { parseZitadelOidcConfig } from '@nabe/auth';
import { healthResponseSchema, placeholderRouteResponseSchema } from '@nabe/validation';

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
  ...healthResponseSchema.parse({
    status: 'ok',
    service: 'nabe-api'
  })
}));

for (const route of placeholderRoutes) {
  app.get(`/v1/${route}`, async () => ({
    ...placeholderRouteResponseSchema.parse({
      route,
      status: 'placeholder'
    })
  }));
}

if (process.env.OIDC_ISSUER_URL && process.env.OIDC_CLIENT_ID) {
  parseZitadelOidcConfig({
    issuerUrl: process.env.OIDC_ISSUER_URL,
    clientId: process.env.OIDC_CLIENT_ID,
    clientSecret: process.env.OIDC_CLIENT_SECRET,
    redirectUri: process.env.OIDC_REDIRECT_URI
  });
}

const host = process.env.NABE_API_HOST ?? '0.0.0.0';
const port = Number(process.env.NABE_API_PORT ?? 8080);

await app.listen({ host, port });
