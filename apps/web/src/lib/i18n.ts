export type Language = 'de' | 'en';

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
    subtitle: 'Zentrale Steuerung für Cloud DNS, Geräte und zukünftige Edge Nodes.',
    language: 'Sprache',
    theme: 'Darstellung',
    system: 'System',
    light: 'Hell',
    dark: 'Dunkel',
    statusCopy: 'Cloud DNS ist als Platzhalter verbunden.',
    devicesCopy: 'Device Clients und Client Tokens werden hier verwaltet.',
    queriesCopy: 'Abfragen werden serverseitig nach Besitz gefiltert.',
    spokesCopy: 'Speiche Agenten melden später Edge Nodes an.',
    instancesCopy: 'DNS Engines werden über Adapter angebunden.'
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
    subtitle: 'Central control for Cloud DNS, devices, and future Edge Nodes.',
    language: 'Language',
    theme: 'Theme',
    system: 'System',
    light: 'Light',
    dark: 'Dark',
    statusCopy: 'Cloud DNS is connected as a placeholder.',
    devicesCopy: 'Device Clients and Client Tokens will be managed here.',
    queriesCopy: 'Queries are filtered server-side by ownership.',
    spokesCopy: 'Speiche Agents will enroll Edge Nodes later.',
    instancesCopy: 'DNS Engines are attached through adapters.'
  }
} as const;

export function detectLanguage(): Language {
  if (typeof navigator === 'undefined') {
    return 'de';
  }

  return navigator.language.toLowerCase().startsWith('en') ? 'en' : 'de';
}
