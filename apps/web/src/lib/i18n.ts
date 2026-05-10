import { languageSchema, type Language } from '@nabe/validation';

export type { Language };

export const messages = {
  de: {
    overview: 'Übersicht',
    dnsStatus: 'DNS-Status',
    myDevices: 'Meine Geräte',
    queries: 'Abfragen',
    spokes: 'Speichen',
    instances: 'Instanzen',
    settings: 'Einstellungen',
    title: 'Nabe Console',
    subtitle: 'Zentrale Steuerung für Cloud DNS und die verbundene DNS Engine.',
    language: 'Sprache',
    theme: 'Darstellung',
    system: 'System',
    light: 'Hell',
    dark: 'Dunkel',
    login: 'Mit Zitadel anmelden',
    logout: 'Abmelden',
    role: 'Rolle',
    connected: 'Verbunden',
    disconnected: 'Nicht verbunden',
    notAuthenticated: 'Nicht angemeldet',
    engineVersion: 'Version',
    protection: 'Schutz',
    queryLog: 'Abfrageprotokoll',
    statistics: 'Statistiken',
    enabled: 'Aktiv',
    disabled: 'Inaktiv',
    unavailable: 'Nicht verfügbar',
    dnsQueries: 'DNS-Abfragen',
    blockedQueries: 'Blockiert',
    statusCopy: 'Live-Status der verbundenen AdGuard Home DNS Engine.',
    devicesCopy: 'Device Clients und Client Tokens kommen nach dem Engine-MVP.',
    queriesCopy: 'Abfragen werden später serverseitig nach Besitz gefiltert.',
    spokesCopy: 'Speiche bleibt für MVP 0 außerhalb des Deployments.',
    instancesCopy: 'Cloud DNS läuft im lokalen Compose-Stack.'
  },
  en: {
    overview: 'Overview',
    dnsStatus: 'DNS Status',
    myDevices: 'My Devices',
    queries: 'Queries',
    spokes: 'Spokes',
    instances: 'Instances',
    settings: 'Settings',
    title: 'Nabe Console',
    subtitle: 'Central control for Cloud DNS and the connected DNS Engine.',
    language: 'Language',
    theme: 'Theme',
    system: 'System',
    light: 'Light',
    dark: 'Dark',
    login: 'Sign in with Zitadel',
    logout: 'Sign out',
    role: 'Role',
    connected: 'Connected',
    disconnected: 'Disconnected',
    notAuthenticated: 'Not signed in',
    engineVersion: 'Version',
    protection: 'Protection',
    queryLog: 'Query log',
    statistics: 'Statistics',
    enabled: 'Enabled',
    disabled: 'Disabled',
    unavailable: 'Unavailable',
    dnsQueries: 'DNS queries',
    blockedQueries: 'Blocked',
    statusCopy: 'Live status of the connected AdGuard Home DNS Engine.',
    devicesCopy: 'Device Clients and Client Tokens come after the engine MVP.',
    queriesCopy: 'Queries will later be filtered server-side by ownership.',
    spokesCopy: 'Speiche stays outside the MVP 0 deployment.',
    instancesCopy: 'Cloud DNS runs in the local Compose stack.'
  }
} as const;

export function detectLanguage(): Language {
  if (typeof navigator === 'undefined') {
    return 'de';
  }

  return languageSchema.parse(navigator.language.toLowerCase().startsWith('en') ? 'en' : 'de');
}
