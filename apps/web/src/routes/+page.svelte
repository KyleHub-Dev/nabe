<script lang="ts">
  import { onMount } from 'svelte';
  import { detectLanguage, messages, type Language } from '$lib/i18n';
  import { languageSchema, themeSchema, type Theme } from '@nabe/validation';

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
      kind: string;
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
  };

  let language: Language = 'de';
  let theme: Theme = 'system';
  let dashboard: Dashboard | null = null;
  let dashboardError = '';
  const apiUrl = import.meta.env.VITE_NABE_API_URL ?? 'http://localhost:8080';

  $: t = messages[language];

  const cards = [
    { key: 'dnsStatus', body: 'statusCopy' },
    { key: 'myDevices', body: 'devicesCopy' },
    { key: 'queries', body: 'queriesCopy' },
    { key: 'spokes', body: 'spokesCopy' },
    { key: 'instances', body: 'instancesCopy' }
  ] as const;

  onMount(() => {
    language = languageSchema.safeParse(localStorage.getItem('nabe-language')).data ?? detectLanguage();
    theme = themeSchema.safeParse(localStorage.getItem('nabe-theme')).data ?? 'system';
    applyTheme(theme);
    void loadDashboard();
  });

  function setLanguage(next: Language) {
    language = next;
    localStorage.setItem('nabe-language', next);
  }

  function setTheme(next: Theme) {
    theme = next;
    localStorage.setItem('nabe-theme', next);
    applyTheme(next);
  }

  function applyTheme(next: Theme) {
    document.documentElement.dataset.theme = next;
  }

  async function loadDashboard() {
    dashboardError = '';
    try {
      const response = await fetch(`${apiUrl}/api/dashboard`, { credentials: 'include' });
      if (response.status === 401) {
        dashboard = null;
        return;
      }
      if (!response.ok) {
        dashboard = null;
        dashboardError = `${response.status}`;
        return;
      }
      dashboard = await response.json();
    } catch (error) {
      dashboard = null;
      dashboardError = error instanceof Error ? error.message : 'unknown';
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
  <title>Nabe Console</title>
</svelte:head>

<main class="shell">
  <aside class="sidebar">
    <div class="brand">Nabe</div>
    <nav aria-label="Primary">
      <a href="/" aria-current="page">{t.overview}</a>
      <a href="/">{t.dnsStatus}</a>
      <a href="/">{t.myDevices}</a>
      <a href="/">{t.queries}</a>
      <a href="/">{t.spokes}</a>
      <a href="/">{t.instances}</a>
      <a href="/">{t.settings}</a>
    </nav>
  </aside>

  <section class="content">
    <header class="topbar">
      <div>
        <h1>{t.title}</h1>
        <p>{t.subtitle}</p>
      </div>

      <div class="controls">
        {#if dashboard}
          <button class="text-button" type="button" on:click={signOut}>{t.logout}</button>
        {:else}
          <button class="text-button" type="button" on:click={signIn}>{t.login}</button>
        {/if}

        <label>
          <span>{t.language}</span>
          <select bind:value={language} on:change={(event) => setLanguage(event.currentTarget.value as Language)}>
            <option value="de">Deutsch</option>
            <option value="en">English</option>
          </select>
        </label>

        <label>
          <span>{t.theme}</span>
          <select bind:value={theme} on:change={(event) => setTheme(event.currentTarget.value as Theme)}>
            <option value="system">{t.system}</option>
            <option value="light">{t.light}</option>
            <option value="dark">{t.dark}</option>
          </select>
        </label>
      </div>
    </header>

    <section class="status-band">
      <div>
        <span>{t.dnsStatus}</span>
        <strong>{dashboard?.engine.connected ? t.connected : t.disconnected}</strong>
      </div>
      <div>
        <span>{dashboard?.principal.email ?? dashboard?.principal.displayName ?? t.notAuthenticated}</span>
        <strong>{dashboard ? `${t.role}: ${dashboard.principal.roles.join(', ')}` : dashboardError || t.unavailable}</strong>
      </div>
    </section>

    <div class="grid">
      {#if dashboard}
        <article class="card">
          <h2>{dashboard.engine.name}</h2>
          <dl>
            <div><dt>{t.engineVersion}</dt><dd>{dashboard.engine.version ?? t.unavailable}</dd></div>
            <div><dt>{t.protection}</dt><dd>{dashboard.engine.protectionEnabled ? t.enabled : t.disabled}</dd></div>
            <div><dt>{t.queryLog}</dt><dd>{dashboard.engine.queryLogEnabled ? t.enabled : t.disabled}</dd></div>
            <div><dt>{t.statistics}</dt><dd>{dashboard.engine.statisticsEnabled ? t.enabled : t.disabled}</dd></div>
          </dl>
        </article>

        <article class="card">
          <h2>{t.queries}</h2>
          <dl>
            <div><dt>{t.dnsQueries}</dt><dd>{dashboard.stats.available ? dashboard.stats.dnsQueries : t.unavailable}</dd></div>
            <div><dt>{t.blockedQueries}</dt><dd>{dashboard.stats.available ? dashboard.stats.blockedFiltering : t.unavailable}</dd></div>
          </dl>
        </article>
      {/if}

      {#each cards as card}
        <article class="card">
          <h2>{t[card.key]}</h2>
          <p>{t[card.body]}</p>
        </article>
      {/each}
    </div>
  </section>
</main>
