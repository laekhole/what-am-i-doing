// Run with: node tests/dashboard.cjs. Executes the actual inline dashboard script.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const html = fs.readFileSync(require('node:path').join(__dirname, '../src/default.html'), 'utf8');
const script = html.slice(html.lastIndexOf('<script>') + 8, html.lastIndexOf('</script>'));

function dashboard(protocol = 'http:') {
  let source, live = { className: '' }, replaces = 0;
  const requests = [], timers = new Map();
  const root = { replaceChildren() { replaces++; live = { className: '' }; } };
  const form = { hidden: true };
  let nextTimer = 0;
  class EventSource {
    constructor() { source = this; this.readyState = 1; }
  }
  vm.runInNewContext(script, {
    location: { protocol, hostname: '127.0.0.1', search: '?lang=en' },
    document: {
      getElementById: id => id === 'live' ? live : form,
      querySelector: () => root,
    },
    window: { EventSource }, EventSource,
    DOMParser: class {
      parseFromString(body) {
        return { querySelector: () => body === 'valid' ? { childNodes: [] } : null };
      }
    },
    fetch: (url, options) => new Promise((resolve, reject) => requests.push({ url, options, resolve, reject })),
    setTimeout: callback => { timers.set(++nextTimer, callback); return nextTimer; },
    clearTimeout: id => timers.delete(id),
  });
  return {
    source, requests, timers, form,
    get live() { return live.className; },
    get replaces() { return replaces; },
    respond(index, ok = true, body = 'valid') { requests[index].resolve({ ok, text: async () => body }); },
    retry() { const [id, callback] = timers.entries().next().value; timers.delete(id); callback(); },
  };
}

// Flush the fetch/text/finally promise chain, without real retry delays.
const settle = () => new Promise(resolve => setImmediate(resolve));

(async () => {
  const staticPage = dashboard('file:');
  assert.equal(staticPage.source, undefined);
  assert.equal(staticPage.form.hidden, true);

  const page = dashboard();
  page.source.onopen();
  assert.notEqual(page.live, 'on', 'connection alone must not imply fresh content');
  assert.equal(page.requests[0].url, '/?lang=en');
  page.respond(0);
  await settle();
  assert.equal(page.live, 'on');
  assert.equal(page.replaces, 1);

  for (const failure of ['network', 'http', 'missing-root']) {
    page.source.onmessage();
    const request = page.requests.length - 1;
    if (failure === 'network') page.requests[request].reject(new Error('offline'));
    else page.respond(request, failure !== 'http', failure === 'missing-root' ? 'invalid' : 'valid');
    await settle();
    assert.equal(page.live, 'off', failure);
    const replaces = page.replaces;
    assert.equal(page.timers.size, 1, 'failed refresh must retry even without another SSE change');
    page.retry();
    page.respond(page.requests.length - 1);
    await settle();
    assert.equal(page.live, 'on');
    assert.equal(page.replaces, replaces + 1);
    assert.equal(page.timers.size, 0);
  }

  page.source.onmessage();
  let request = page.requests.length - 1;
  page.source.onmessage();
  page.source.onmessage();
  assert.equal(page.requests.length, request + 1, 'only one refresh may be in flight');
  page.respond(request);
  await settle();
  assert.equal(page.requests.length, request + 2, 'coalesce ticks into one following refresh');
  page.source.readyState = 0;
  page.source.onerror();
  page.respond(request + 1);
  await settle();
  assert.equal(page.live, 'off', 'an in-flight response must not hide an SSE disconnect');
  page.source.readyState = 1;
  page.source.onopen();
  page.respond(page.requests.length - 1);
  await settle();
  assert.equal(page.live, 'on');
  console.log('dashboard refresh regressions passed');
})().catch(error => { console.error(error); process.exitCode = 1; });
