import { z } from 'zod';

export const languageSchema = z.enum(['de', 'en']);
export const themeSchema = z.enum(['system', 'light', 'dark']);

export const healthResponseSchema = z.object({
  status: z.literal('ok'),
  service: z.string().min(1)
});

export const placeholderRouteResponseSchema = z.object({
  route: z.string().min(1),
  status: z.literal('placeholder')
});

export const deviceClientSchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(1),
  clientTokenHint: z.string().min(1).optional()
});

export type Language = z.infer<typeof languageSchema>;
export type Theme = z.infer<typeof themeSchema>;
export type HealthResponse = z.infer<typeof healthResponseSchema>;
export type PlaceholderRouteResponse = z.infer<typeof placeholderRouteResponseSchema>;
export type DeviceClient = z.infer<typeof deviceClientSchema>;
