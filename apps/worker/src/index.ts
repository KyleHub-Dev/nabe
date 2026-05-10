const jobs = [
  'sync stats',
  'poll AdGuard',
  'process Speiche heartbeats',
  'cleanup revoked tokens'
];

console.info('Nabe worker started', {
  service: 'nabe-worker',
  jobs
});
