import { z } from 'zod';

export const zitadelOidcConfigSchema = z.object({
  issuerUrl: z.string().url(),
  clientId: z.string().min(1),
  clientSecret: z.string().optional(),
  redirectUri: z.string().url().optional()
});

export type ZitadelOidcConfig = z.infer<typeof zitadelOidcConfigSchema>;

export function parseZitadelOidcConfig(input: unknown): ZitadelOidcConfig {
  return zitadelOidcConfigSchema.parse(input);
}
