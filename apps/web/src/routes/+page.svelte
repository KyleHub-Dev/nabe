<script lang="ts">
  import { onMount } from 'svelte';
  import { detectLanguage, messages, type Language } from '$lib/i18n';
  import { languageSchema, themeSchema, type Theme } from '@nabe/validation';

  let language: Language = 'de';
  let theme: Theme = 'system';

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

    <div class="grid">
      {#each cards as card}
        <article class="card">
          <h2>{t[card.key]}</h2>
          <p>{t[card.body]}</p>
        </article>
      {/each}
    </div>
  </section>
</main>
