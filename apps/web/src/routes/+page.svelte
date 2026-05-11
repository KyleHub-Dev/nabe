<script lang="ts">
  import { onMount } from 'svelte';
  import { languageSchema, themeSchema, type Theme } from '@nabe/validation';
  import { detectLanguage, type Language } from '$lib/i18n';

  type Principal = {
    provider: string;
    subject: string;
    email?: string;
    displayName?: string;
    roles: string[];
  };

  type Dashboard = {
    principal: Principal;
    engine: {
      name: string;
      connected: boolean;
      version?: string;
      protectionEnabled: boolean;
      queryLogEnabled: boolean;
      statisticsEnabled: boolean;
      lastCheckedAt: string;
    };
    stats: {
      available: boolean;
      dnsQueries: number;
      blockedFiltering: number;
    };
    adguard: {
      url: string;
      username?: string;
      password?: string;
    };
  };

  type View = 'overview' | 'cloudDns';

  let language: Language = 'de';
  let theme: Theme = 'system';
  let view: View = 'overview';
  let dashboard: Dashboard | null = null;
  let loading = true;
  let dashboardError = '';
  const apiUrl = import.meta.env.VITE_NABE_API_URL ?? 'http://localhost:8080';
  const copy = {
    de: {
      eyebrow: '01/KyleHub DNS Control Plane',
      lead: 'Eine ruhige Konsole für Cloud-DNS, Device Clients und die verbundene DNS Engine.',
      login: 'Mit Zitadel anmelden',
      checking: 'Session wird geprüft',
      overview: 'Übersicht',
      open: 'Öffnen',
      signedIn: 'Angemeldet',
      status: 'Status',
      connected: 'Verbunden',
      disconnected: 'Getrennt',
      dnsQueries: 'DNS-Abfragen',
      blocked: 'Blockiert',
      blockRate: 'Blockrate',
      protection: 'Schutz',
      queryLog: 'Query Log',
      statistics: 'Statistiken',
      active: 'Aktiv',
      inactive: 'Inaktiv',
      checked: 'Zuletzt geprüft',
      devAccess: 'Dev Zugang',
      local: 'lokal',
      user: 'Benutzer',
      password: 'Passwort',
      roles: 'Rollen',
      logout: 'Abmelden',
      console: 'Konsole',
      themeLabel: 'Darstellung wechseln'
    },
    en: {
      eyebrow: '01/KyleHub DNS Control Plane',
      lead: 'A quiet console for Cloud DNS, Device Clients, and the connected DNS Engine.',
      login: 'Sign in with Zitadel',
      checking: 'Checking session',
      overview: 'Overview',
      open: 'Open',
      signedIn: 'Signed in',
      status: 'Status',
      connected: 'Connected',
      disconnected: 'Disconnected',
      dnsQueries: 'DNS queries',
      blocked: 'Blocked',
      blockRate: 'Block rate',
      protection: 'Protection',
      queryLog: 'Query log',
      statistics: 'Statistics',
      active: 'Active',
      inactive: 'Inactive',
      checked: 'Last checked',
      devAccess: 'Dev access',
      local: 'local',
      user: 'User',
      password: 'Password',
      roles: 'Roles',
      logout: 'Sign out',
      console: 'Console',
      themeLabel: 'Switch theme'
    }
  } as const;

  $: blockedRate =
    dashboard?.stats.available && dashboard.stats.dnsQueries > 0
      ? Math.round((dashboard.stats.blockedFiltering / dashboard.stats.dnsQueries) * 100)
      : 0;
  $: userLabel =
    dashboard?.principal.displayName ?? dashboard?.principal.email ?? dashboard?.principal.subject ?? '';
  $: text = copy[language];
  $: nextTheme = (theme === 'dark' ? 'light' : 'dark') as Theme;
  $: activeClients = dashboard?.stats.available && dashboard.stats.dnsQueries > 0 ? 1 : 0;

  onMount(() => {
    language = languageSchema.safeParse(localStorage.getItem('nabe-language')).data ?? detectLanguage();
    theme = themeSchema.safeParse(localStorage.getItem('nabe-theme')).data ?? 'system';
    applyTheme(theme);
    void loadDashboard();
  });

  function setTheme(next: Theme) {
    theme = next;
    localStorage.setItem('nabe-theme', next);
    applyTheme(next);
  }

  function setLanguage(next: Language) {
    language = next;
    localStorage.setItem('nabe-language', next);
  }

  function applyTheme(next: Theme) {
    document.documentElement.dataset.theme = next;
  }

  async function loadDashboard() {
    loading = true;
    dashboardError = '';
    try {
      const response = await fetch(`${apiUrl}/api/dashboard`, { credentials: 'include' });
      if (response.status === 401) {
        dashboard = null;
        return;
      }
      if (!response.ok) {
        dashboard = null;
        dashboardError = `API ${response.status}`;
        return;
      }
      dashboard = await response.json();
    } catch (error) {
      dashboard = null;
      dashboardError = error instanceof Error ? error.message : 'unbekannter Fehler';
    } finally {
      loading = false;
    }
  }

  function signIn() {
    window.location.href = `${apiUrl}/auth/login`;
  }

  async function signOut() {
    await fetch(`${apiUrl}/auth/logout`, { method: 'POST', credentials: 'include' });
    dashboard = null;
  }
</script>

<svelte:head>
  <title>{dashboard ? `Nabe ${text.console}` : 'Nabe'}</title>
</svelte:head>

{#if !dashboard}
  <main class="landing">
    <header class="landing-bar">
      <strong class="wordmark">Nabe</strong>
      <div class="chrome">
        <div class="language-tray" aria-label="Language">
          <button class:active={language === 'de'} type="button" on:click={() => setLanguage('de')}>DE</button>
          <button class:active={language === 'en'} type="button" on:click={() => setLanguage('en')}>EN</button>
        </div>
        <button class="theme-button" type="button" aria-label={text.themeLabel} on:click={() => setTheme(nextTheme)}>
          {theme === 'dark' ? '☼' : '●'}
        </button>
      </div>
    </header>

    <section class="landing-center">
      <p class="eyebrow">{text.eyebrow}</p>
      <h1>Nabe</h1>
      <p class="lead">{text.lead}</p>
      <div class="landing-actions">
        <button class="primary-button" type="button" on:click={signIn}>{text.login}</button>
        {#if loading}
          <span>{text.checking}</span>
        {:else if dashboardError}
          <span>{dashboardError}</span>
        {/if}
      </div>
    </section>
  </main>
{:else}
  <main class="console">
    <aside class="rail">
      <div class="mark wordmark">Nabe</div>
      <nav aria-label={text.console}>
        <button type="button" class:active={view === 'overview'} on:click={() => (view = 'overview')}>{text.overview}</button>
        <button type="button" class:active={view === 'cloudDns'} on:click={() => (view = 'cloudDns')}>Cloud-DNS</button>
        <a href={dashboard.adguard.url} target="_blank" rel="noreferrer">AdGuard Debug</a>
      </nav>
      <div class="rail-tools">
        <div class="language-tray" aria-label="Language">
          <button class:active={language === 'de'} type="button" on:click={() => setLanguage('de')}>DE</button>
          <button class:active={language === 'en'} type="button" on:click={() => setLanguage('en')}>EN</button>
        </div>
        <button class="theme-button" type="button" aria-label={text.themeLabel} on:click={() => setTheme(nextTheme)}>
          {theme === 'dark' ? '☼' : '●'}
        </button>
      </div>
    </aside>

    <section class="workspace">
      <header class="console-top">
        <div>
          <p class="eyebrow">{view === 'overview' ? '01/Übersicht' : '02/Cloud-DNS'}</p>
          <h1>{view === 'overview' ? text.overview : 'Cloud-DNS'}</h1>
        </div>
        <div class="session">
          <span>{text.signedIn}: {userLabel}</span>
          <button class="ghost-button" type="button" on:click={signOut}>{text.logout}</button>
        </div>
      </header>

      {#if view === 'overview'}
        <section class="metrics" aria-label="Globale Statistiken">
          <article>
            <span>{text.status}</span>
            <strong>{dashboard.engine.connected ? text.connected : text.disconnected}</strong>
          </article>
          <article>
            <span>{text.dnsQueries}</span>
            <strong>{dashboard.stats.available ? dashboard.stats.dnsQueries.toLocaleString('de-DE') : 'n/a'}</strong>
          </article>
          <article>
            <span>{text.blocked}</span>
            <strong>{dashboard.stats.available ? dashboard.stats.blockedFiltering.toLocaleString('de-DE') : 'n/a'}</strong>
          </article>
          <article>
            <span>Geräte aktiv</span>
            <strong>{activeClients}</strong>
          </article>
        </section>

        <section class="overview-grid">
          <article class="panel wide">
            <div class="panel-head">
              <h2>Systemüberblick</h2>
              <span>{new Date(dashboard.engine.lastCheckedAt).toLocaleString(language === 'de' ? 'de-DE' : 'en-US')}</span>
            </div>
            <div class="signal-list">
              <div><span>Cloud-DNS</span><strong>{dashboard.engine.connected ? 'online' : 'offline'}</strong></div>
              <div><span>Schutz</span><strong>{dashboard.engine.protectionEnabled ? 'aktiv' : 'inaktiv'}</strong></div>
              <div><span>Blockrate</span><strong>{dashboard.stats.available ? `${blockedRate}%` : 'n/a'}</strong></div>
              <div><span>Query Log</span><strong>{dashboard.engine.queryLogEnabled ? 'aktiv' : 'inaktiv'}</strong></div>
            </div>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Geräte</h2>
              <span>mock</span>
            </div>
            <ul class="plain-list">
              <li><span>Kyles Laptop</span><strong>Cloud-DNS</strong></li>
              <li><span>Telefon</span><strong>wartet auf Token</strong></li>
              <li><span>Home Router</span><strong>geplant</strong></li>
            </ul>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Nächste Aufgaben</h2>
              <span>MVP</span>
            </div>
            <ul class="plain-list">
              <li><span>Device Clients modellieren</span><strong>API</strong></li>
              <li><span>Rollen pro Aktion prüfen</span><strong>Auth</strong></li>
              <li><span>Query-Log Besitz filtern</span><strong>Privacy</strong></li>
            </ul>
          </article>
        </section>
      {:else}
        <section class="metrics" aria-label="Cloud-DNS Statistiken">
          <article>
            <span>{text.protection}</span>
            <strong>{dashboard.engine.protectionEnabled ? text.active : text.inactive}</strong>
          </article>
          <article>
            <span>{text.statistics}</span>
            <strong>{dashboard.engine.statisticsEnabled ? text.active : text.inactive}</strong>
          </article>
          <article>
            <span>{text.queryLog}</span>
            <strong>{dashboard.engine.queryLogEnabled ? text.active : text.inactive}</strong>
          </article>
          <article>
            <span>Engine</span>
            <strong>AdGuard</strong>
          </article>
        </section>

        <section class="cloud-grid">
          <article class="panel">
            <div class="panel-head">
              <h2>Filter & Schutz</h2>
              <span>mock</span>
            </div>
            <ul class="plain-list">
              <li><span>DNS-Filter</span><strong>aktiv</strong></li>
              <li><span>Safe Browsing</span><strong>aus</strong></li>
              <li><span>Parental Control</span><strong>aus</strong></li>
              <li><span>Allowlist / Blocklist</span><strong>geplant</strong></li>
            </ul>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Clients & Tokens</h2>
              <span>RBAC</span>
            </div>
            <ul class="plain-list">
              <li><span>Persistent Clients</span><strong>lesen</strong></li>
              <li><span>ClientID Tokens</span><strong>erstellen</strong></li>
              <li><span>Besitzregeln</span><strong>Nabe API</strong></li>
              <li><span>Query Sichtbarkeit</span><strong>serverseitig</strong></li>
            </ul>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Upstream & DNS</h2>
              <span>Unbound</span>
            </div>
            <dl>
              <div><dt>Upstream</dt><dd>172.30.10.10:53</dd></div>
              <div><dt>DoH</dt><dd>intern vorbereitet</dd></div>
              <div><dt>DoT</dt><dd>intern vorbereitet</dd></div>
              <div><dt>Plain DNS</dt><dd>nicht am Host veröffentlicht</dd></div>
            </dl>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>{text.devAccess}</h2>
              <a class="inline-link" href={dashboard.adguard.url} target="_blank" rel="noreferrer">{text.open}</a>
            </div>
            <dl>
              <div><dt>URL</dt><dd><a class="inline-link" href={dashboard.adguard.url} target="_blank" rel="noreferrer">{dashboard.adguard.url}</a></dd></div>
              <div><dt>{text.user}</dt><dd>{dashboard.adguard.username ?? 'n/a'}</dd></div>
              <div><dt>{text.password}</dt><dd>{dashboard.adguard.password ?? 'n/a'}</dd></div>
              <div><dt>{text.roles}</dt><dd>{dashboard.principal.roles.join(', ')}</dd></div>
            </dl>
          </article>
        </section>
      {/if}
    </section>
  </main>
{/if}
