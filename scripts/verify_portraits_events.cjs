#!/usr/bin/env node
// Hidden-browser review against the actual checkout. All HTTP requests are
// fulfilled from the checkout or its existing Release shared assets; external
// hosts and every non-GET method are blocked.

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');

const projectRoot = path.resolve(__dirname, '..');
const workspaceRoot = path.resolve(projectRoot, '..');
const releaseRoot = path.join(workspaceRoot, 'Release');
const verificationRoot = path.join(projectRoot, 'docs', 'verification');
const origin = 'http://127.0.0.1';
const viewport = { width: 1920, height: 1080 };

function usage() {
  console.log(`Usage: node scripts/verify_portraits_events.cjs --wasm <fresh-wasm> [--profile all|notifications|portraits]

Requires the real WASM output from the current Kestrum checkout. Writes stable
headless screenshots under docs/verification. The browser uses a fresh isolated
context per profile and imports the durable native fixtures through Kestrum's
ordinary title-screen Import Campaign control.
`);
}

function argumentsFrom(argv) {
  const options = { profile: 'all' };
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    if (key === '--help' || key === '-h') {
      usage();
      process.exit(0);
    }
    if (key === '--wasm' || key === '--profile') {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) throw new Error(`${key} needs a value`);
      options[key.slice(2)] = value;
      index += 1;
      continue;
    }
    throw new Error(`Unknown argument: ${key}`);
  }
  if (!options.wasm) throw new Error('Pass --wasm with the fresh actual-checkout WASM output');
  if (!['all', 'notifications', 'portraits'].includes(options.profile)) {
    throw new Error('--profile must be all, notifications, or portraits');
  }
  options.wasm = path.resolve(projectRoot, options.wasm);
  if (!fs.existsSync(options.wasm)) throw new Error(`WASM file not found: ${options.wasm}`);
  return options;
}

function contentType(file) {
  switch (path.extname(file).toLowerCase()) {
    case '.html': return 'text/html; charset=utf-8';
    case '.js': case '.mjs': return 'text/javascript; charset=utf-8';
    case '.css': return 'text/css; charset=utf-8';
    case '.wasm': return 'application/wasm';
    case '.json': return 'application/json; charset=utf-8';
    case '.png': return 'image/png';
    case '.jpg': case '.jpeg': return 'image/jpeg';
    case '.webp': return 'image/webp';
    case '.svg': return 'image/svg+xml';
    case '.woff': return 'font/woff';
    case '.woff2': return 'font/woff2';
    case '.ttf': return 'font/ttf';
    case '.ogg': return 'audio/ogg';
    case '.mp3': return 'audio/mpeg';
    default: return 'application/octet-stream';
  }
}

function localFileFor(urlPath, wasmFile) {
  let pathname;
  try {
    pathname = decodeURIComponent(urlPath);
  } catch {
    return null;
  }
  if (pathname === '/kestrum' || pathname === '/kestrum/') {
    return path.join(projectRoot, 'dist', 'webgl', 'index.html');
  }
  if (pathname === '/kestrum/index.html') {
    return path.join(projectRoot, 'dist', 'webgl', 'index.html');
  }
  if (pathname === '/kestrum/kestrum.wasm') return wasmFile;

  const roots = [
    ['/kestrum/assets/', path.join(projectRoot, 'assets')],
    ['/shared-assets/', path.join(releaseRoot, 'shared-assets')],
    ['/release-assets/', path.join(releaseRoot, 'shared-assets')],
  ];
  for (const [prefix, root] of roots) {
    if (!pathname.startsWith(prefix)) continue;
    const suffix = pathname.slice(prefix.length);
    const file = path.resolve(root, suffix);
    const rootPrefix = `${path.resolve(root)}${path.sep}`;
    return file.startsWith(rootPrefix) ? file : null;
  }

  for (const name of ['shared.css', 'bug-report.css', 'bug-report.js']) {
    if (pathname === `/${name}`) return path.join(releaseRoot, name);
  }
  if (pathname === '/favicon.ico') return '';
  return null;
}

function installRoutes(context, wasmFile, audit) {
  return context.route('**/*', async (route) => {
    const request = route.request();
    let requestUrl;
    try {
      requestUrl = new URL(request.url());
    } catch {
      audit.unmapped.push(`invalid URL: ${request.url()}`);
      await route.abort('blockedbyclient');
      return;
    }
    if (requestUrl.origin !== origin) {
      audit.blockedExternal.push(`${request.method()} ${requestUrl.origin}`);
      await route.abort('blockedbyclient');
      return;
    }
    if (request.method() !== 'GET') {
      audit.blockedMethods.push(`${request.method()} ${requestUrl.pathname}`);
      await route.fulfill({ status: 405, body: 'Headless review permits GET only.' });
      return;
    }
    const file = localFileFor(requestUrl.pathname, wasmFile);
    if (file === '') {
      await route.fulfill({ status: 204, body: '' });
      return;
    }
    if (!file || !fs.existsSync(file) || !fs.statSync(file).isFile()) {
      audit.unmapped.push(requestUrl.pathname);
      await route.fulfill({ status: 404, body: `No checkout asset for ${requestUrl.pathname}` });
      return;
    }
    await route.fulfill({
      status: 200,
      path: file,
      headers: {
        'content-type': contentType(file),
        'cache-control': 'no-store',
        'x-content-type-options': 'nosniff',
      },
    });
  });
}

async function newReviewPage(browser, fixturePath, audit) {
  if (!fs.existsSync(fixturePath)) throw new Error(`Durable native fixture missing: ${fixturePath}`);
  const fixture = fs.readFileSync(fixturePath, 'utf8');
  JSON.parse(fixture);
  const context = await browser.newContext({
    viewport,
    deviceScaleFactor: 1,
    serviceWorkers: 'block',
  });
  await context.addInitScript(({ key, value }) => {
    try {
      localStorage.setItem(key, value);
    } catch (error) {
      window.__kestrumFixtureStorageError = String(error);
    }
    const fitCanvasWithoutFullscreen = () => {
      if (!document.body) return;
      document.body.classList.add('game-playing');
      if (window.wasm_exports) window.dispatchEvent(new Event('resize'));
    };
    const root = document.documentElement;
    if (root) {
      const watch = new MutationObserver(fitCanvasWithoutFullscreen);
      watch.observe(root, { childList: true, subtree: true });
    }
    document.addEventListener('DOMContentLoaded', () => {
      fitCanvasWithoutFullscreen();
      requestAnimationFrame(() => window.dispatchEvent(new Event('resize')));
    }, { once: true });
  }, { key: 'kestrum_save_kestrum_strategic_v2', value: fixture });
  await installRoutes(context, options.wasm, audit);
  const page = await context.newPage();
  page.on('pageerror', (error) => audit.pageErrors.push(error.stack || error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') audit.consoleErrors.push(message.text());
  });
  page.on('requestfailed', (request) => {
    const url = request.url();
    if (url.startsWith(origin)) audit.failedRequests.push(`${request.method()} ${url}: ${request.failure()?.errorText || 'failed'}`);
  });
  await page.goto(`${origin}/kestrum/index.html`, { waitUntil: 'domcontentloaded', timeout: 60000 });
  await page.waitForFunction(() => Boolean(window.wasm_exports), { timeout: 60000 });
  await page.waitForFunction(() => {
    const canvas = document.querySelector('#glcanvas');
    return canvas
      && canvas.clientWidth >= 1900
      && canvas.clientHeight >= 1060
      && canvas.width === 1920
      && canvas.height === 1080;
  }, { timeout: 15000 });
  const dimensions = await page.evaluate(() => {
    const canvas = document.querySelector('#glcanvas');
    const rect = canvas.getBoundingClientRect();
    return {
      viewport: [window.innerWidth, window.innerHeight],
      canvasCss: [Math.round(rect.width), Math.round(rect.height)],
      canvasPixels: [canvas.width, canvas.height],
      devicePixelRatio: window.devicePixelRatio,
      fullScreenElement: Boolean(document.fullscreenElement),
      playModeClass: document.body.classList.contains('game-playing'),
      fixtureError: window.__kestrumFixtureStorageError || null,
    };
  });
  if (dimensions.viewport[0] !== 1920 || dimensions.viewport[1] !== 1080
      || dimensions.canvasPixels[0] !== 1920 || dimensions.canvasPixels[1] !== 1080
      || dimensions.devicePixelRatio !== 1 || dimensions.fullScreenElement
      || !dimensions.playModeClass || dimensions.fixtureError) {
    throw new Error(`Unexpected headless viewport/fixture state: ${JSON.stringify(dimensions)}`);
  }
  await page.waitForTimeout(1200);
  return { context, page, dimensions };
}

async function click(page, x, y, label) {
  await page.mouse.click(x, y, { delay: 40 });
  await page.waitForTimeout(450);
  console.log(`  clicked ${label} at ${x},${y}`);
}

async function scroll(page, x, y, deltaY, label) {
  await page.mouse.move(x, y);
  await page.mouse.wheel(0, deltaY);
  await page.waitForTimeout(500);
  console.log(`  wheel ${label} at ${x},${y} deltaY=${deltaY}`);
}

async function drag(page, fromX, fromY, toX, toY, label) {
  await page.mouse.move(fromX, fromY);
  await page.mouse.down();
  await page.mouse.move(toX, toY, { steps: 8 });
  await page.mouse.up();
  await page.waitForTimeout(500);
  console.log(`  dragged ${label} from ${fromX},${fromY} to ${toX},${toY}`);
}

async function saveFrame(page, filename, audit, label) {
  const output = path.join(verificationRoot, filename);
  const bytes = await page.screenshot({ path: output, type: 'png', fullPage: false, animations: 'disabled' });
  const dimensions = bytes.length >= 24 && bytes.toString('hex', 0, 8) === '89504e470d0a1a0a'
    ? [bytes.readUInt32BE(16), bytes.readUInt32BE(20)]
    : null;
  if (!dimensions || dimensions[0] !== viewport.width || dimensions[1] !== viewport.height) {
    throw new Error(`Capture ${filename} has unexpected PNG dimensions: ${JSON.stringify(dimensions)}`);
  }
  const digest = crypto.createHash('sha256').update(bytes).digest('hex');
  const frame = { label, path: output, width: dimensions[0], height: dimensions[1], bytes: bytes.length, sha256: digest };
  audit.frames.push(frame);
  console.log(`  captured ${filename} ${dimensions[0]}x${dimensions[1]} sha256=${digest}`);
}

async function importCampaign(page) {
  // This is the visible title-screen button in the established 1920x1080 UI.
  await click(page, 1680, 850, 'ordinary Import Campaign');
  await page.waitForTimeout(2500);
}

async function notificationsReview(browser, wasmFile) {
  console.log('Headless notifications review (isolated browser context)');
  const audit = { frames: [], blockedExternal: [], blockedMethods: [], unmapped: [], failedRequests: [], pageErrors: [], consoleErrors: [] };
  const { context, page, dimensions } = await newReviewPage(
    browser,
    path.join(verificationRoot, 'notification_review_save.json'),
    audit,
  );
  console.log(`  canvas: ${JSON.stringify(dimensions)}`);
  await importCampaign(page);
  await saveFrame(page, 'ui_web_notifications.png', audit, 'imported campaign and event rail');
  await click(page, 768, 128, 'Notifications rail');
  await saveFrame(page, 'ui_web_notification_recent.png', audit, 'notification recent card');
  await click(page, 260, 290, 'first dated receipt');
  await saveFrame(page, 'ui_web_notification_details.png', audit, 'dated receipt detail');
  await click(page, 380, 732, 'receipt Settings action');
  await saveFrame(page, 'ui_web_notification_settings.png', audit, 'notification delivery settings');
  await click(page, 205, 290, 'People filter');
  await click(page, 446, 442, 'mute first People event type');
  await saveFrame(page, 'ui_web_notification_muted.png', audit, 'muted People event type');
  await click(page, 430, 676, 'People settings next page');
  await saveFrame(page, 'ui_web_notification_settings_page2.png', audit, 'People settings second page');
  await click(page, 92, 232, 'Recent tab');
  await click(page, 140, 746, 'Mark all notifications read');
  await saveFrame(page, 'ui_web_notifications_read.png', audit, 'read notifications');
  await click(page, 451, 191, 'close notifications card');

  await click(page, 1842, 48, 'campaign Menu');
  await click(page, 1079, 568, 'Menu Settings');
  await click(page, 960, 586, 'global Notification Settings');
  await saveFrame(page, 'ui_web_notification_global_settings.png', audit, 'global notification settings');
  await click(page, 768, 434, 'global All category');
  await click(page, 1146, 621, 'global notification page 2');
  await saveFrame(page, 'ui_web_notification_global_page2.png', audit, 'global settings second event page');
  await click(page, 1160, 484, 'global delivery Off');
  const preferenceBeforeReload = await page.evaluate(() => localStorage.getItem('kestrum_preferences'));
  if (!preferenceBeforeReload) throw new Error('Global notification preference was not persisted to localStorage');
  for (let index = 0; index < 7; index += 1) {
    await click(page, 1152, 434, `global category next ${index + 1}`);
  }
  await saveFrame(page, 'ui_web_notification_global_last_category.png', audit, 'last global settings category');
  await click(page, 1152, 434, 'disabled next category boundary');
  await saveFrame(page, 'ui_web_notification_global_last_category.png', audit, 'last global settings category remains stable');
  await click(page, 960, 754, 'close global notification settings');
  await click(page, 960, 752, 'close Settings overlay');

  await scroll(page, 1050, 520, -240, 'map zoom');
  await saveFrame(page, 'ui_web_events_zoomed.png', audit, 'campaign map after zoom');
  await drag(page, 1170, 560, 1222, 592, 'map pan');
  await saveFrame(page, 'ui_web_events_panned.png', audit, 'campaign map after pan');
  await click(page, 1804, 1032, 'End Turn');
  await page.waitForTimeout(10000);
  await saveFrame(page, 'ui_web_events_after_end_turn.png', audit, 'campaign after End Turn and NPC progress');

  await page.reload({ waitUntil: 'domcontentloaded', timeout: 60000 });
  await page.waitForFunction(() => Boolean(window.wasm_exports), { timeout: 60000 });
  await page.waitForFunction(() => {
    const canvas = document.querySelector('#glcanvas');
    return canvas && canvas.width === 1920 && canvas.height === 1080;
  }, { timeout: 15000 });
  await page.waitForTimeout(2500);
  const persistedPreference = await page.evaluate(() => localStorage.getItem('kestrum_preferences'));
  if (persistedPreference !== preferenceBeforeReload) {
    throw new Error('Global notification preference changed across browser reload');
  }
  const titleBeforeContinue = await page.screenshot({ type: 'png', fullPage: false, animations: 'disabled' });
  await click(page, 480, 506, 'Continue saved campaign after reload');
  await page.waitForTimeout(3000);
  const campaignAfterContinue = await page.screenshot({ type: 'png', fullPage: false, animations: 'disabled' });
  if (titleBeforeContinue.equals(campaignAfterContinue)) {
    throw new Error('Continue did not change the rendered title screen after reload');
  }
  await saveFrame(page, 'ui_web_notifications_reloaded.png', audit, 'campaign reloaded through Continue');
  await context.close();
  return audit;
}

async function portraitsReview(browser, wasmFile) {
  console.log('Headless portraits review (isolated browser context)');
  const audit = { frames: [], blockedExternal: [], blockedMethods: [], unmapped: [], failedRequests: [], pageErrors: [], consoleErrors: [] };
  const { context, page, dimensions } = await newReviewPage(
    browser,
    path.join(verificationRoot, 'portrait_review_save.json'),
    audit,
  );
  console.log(`  canvas: ${JSON.stringify(dimensions)}`);
  await importCampaign(page);
  await saveFrame(page, 'ui_web_portrait_campaign.png', audit, 'imported portrait campaign');
  await click(page, 287, 652, 'selected home Armies control');
  await click(page, 727, 830, 'Army Orders control in centered sheet');
  await click(page, 1240, 614, 'Army People control in centered sheet');
  await saveFrame(page, 'ui_web_portraits.png', audit, 'dense roster portrait sprites');
  await click(page, 951, 400, 'founder Career control in centered sheet');
  await saveFrame(page, 'ui_web_portraits_career.png', audit, 'long-name founder career');
  await context.close();
  return audit;
}

function reportAudit(name, audit) {
  console.log(`${name} audit: ${JSON.stringify({
    frames: audit.frames,
    blockedExternalCount: audit.blockedExternal.length,
    blockedMethods: audit.blockedMethods,
    unmappedLocalRequests: audit.unmapped,
    failedLocalRequests: audit.failedRequests,
    pageErrors: audit.pageErrors,
    consoleErrors: audit.consoleErrors,
  }, null, 2)}`);
  if (audit.blockedMethods.length || audit.unmapped.length || audit.failedRequests.length || audit.pageErrors.length) {
    throw new Error(`${name} browser audit encountered blocked writes, missing local files, failed local requests, or page errors`);
  }
}

let options;
async function main() {
  options = argumentsFrom(process.argv.slice(2));
  if (!fs.existsSync(path.join(projectRoot, 'dist', 'webgl', 'index.html'))) {
    throw new Error('The actual checkout dist/webgl/index.html is missing');
  }
  if (!fs.existsSync(path.join(releaseRoot, 'shared-assets', 'runtime', 'mq_js_bundle.js'))) {
    throw new Error('Existing Release/shared-assets runtime is missing');
  }
  if (!fs.existsSync(verificationRoot)) throw new Error('Durable docs/verification directory is missing');
  const browser = await chromium.launch({
    headless: true,
    args: ['--enable-webgl', '--use-gl=angle', '--use-angle=swiftshader', '--disable-gpu-sandbox'],
  });
  try {
    if (options.profile === 'all' || options.profile === 'notifications') {
      reportAudit('notifications', await notificationsReview(browser, options.wasm));
    }
    if (options.profile === 'all' || options.profile === 'portraits') {
      reportAudit('portraits', await portraitsReview(browser, options.wasm));
    }
  } finally {
    await browser.close();
  }
  console.log('Headless browser review finished. No Fullscreen API or non-GET network requests were issued.');
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exitCode = 1;
});
