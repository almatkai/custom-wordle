// Actual own TS command + locked TypeScript library with recording Raycast boundaries.
// Does not rewrite the implementation, substitute its algorithm or access the site.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { load, ts } = require('./ts-loader.cjs');
const root = path.resolve(__dirname, '..');
let events, toast, fault;
const api = {
  Clipboard: { copy(value) {
    events.push(['copy', value]);
    if (fault === 'copy-throw') throw new Error('recorded copy failure');
    if (fault === 'copy-reject') { const p = Promise.reject(new Error('recorded async copy failure')); p.catch(() => {}); return p; }
    return Promise.resolve();
  } },
  Toast: { Style: { Failure: 'failure', Success: 'success' } },
  async showToast(value) {
    toast = value;
    events.push(['toast', value.style, value.title, value.message ?? null, value.primaryAction?.title ?? null]);
    if (fault === 'toast-reject') throw new Error('recorded toast failure');
  },
  async open(value) { events.push(['open', value]); if (fault === 'open-reject') throw new Error('recorded open failure'); },
};
const command = load(path.join(root, 'fixtures/own/src/create-custom-wordle.ts'), { '@raycast/api': api }).default;
async function actual(input, effectFault) {
  events = []; toast = null; fault = effectFault;
  const word = input.utf16 ? String.fromCharCode(...input.utf16) : input.word;
  let threw = false;
  try { await command({ arguments: { word } }); } catch { threw = true; }
  const output = threw ? { error: 'malformed' } : toast?.style === 'failure' ? { validation: toast.title } : { url: events[0]?.[1] };
  if (!threw && output.url) {
    assert.deepEqual(events.map(e => e[0]), ['copy', 'toast']);
    assert.equal(toast.title, 'Custom Wordle Created');
    assert.equal(toast.message, 'Link copied to clipboard');
    assert.equal(toast.primaryAction.title, 'Open Link');
    assert.equal(events[1][1], 'success');
    try { await toast.primaryAction.onAction(); } catch { threw = true; }
    assert.deepEqual(events.map(e => e[0]), ['copy', 'toast', 'open']);
    assert.equal(events[2][1], output.url);
  } else if (output.validation) {
    assert.deepEqual(events.map(e => e[0]), ['toast']);
    assert.equal(events[0][1], 'failure');
  }
  return { output, events: structuredClone(events), threw };
}
(async () => {
  assert.equal(ts.version, '5.9.3');
  const cases = [{}, ...[null, false, 0, 42, [], {}, ['apple'], '', ' ', 'a', 'APPle', 'verylongword', 'two words', '\ta\tb\t', 'é', 'İ', 'K', 'ſ', '😀', 'a+b', 'a&b', 'a?b', 'a%b', 'a/b', 'a\\b', '\0a', ' a ! ', '\ufeffAPPLE\u2000'].map(word => ({ word }))];
  // Exhaust every individual UTF-16 code unit, including lone surrogates.
  for (let c = 0; c <= 65535; c++) cases.push({ utf16: [c] });
  const whitespace = [9,10,11,12,13,32,160,5760,8192,8193,8194,8195,8196,8197,8198,8199,8200,8201,8202,8232,8233,8239,8287,12288,65279,133,6158,8203];
  for (const c of whitespace) {
    cases.push({ utf16: [c,65,80,80,76,69,c] }, { utf16: [65,c,66] });
  }
  let seed = 0xc0ffee;
  const random = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
  for (let i=0;i<5000;i++) {
    let word=''; const n=1+random()%256;
    for (let j=0;j<n;j++) word+=String.fromCharCode((random()%2?65:97)+random()%26);
    cases.push({ word });
  }
  for (let i=0;i<2000;i++) {
    const c=65536+random()%(0x110000-65536);
    cases.push({ word:String.fromCodePoint(c) });
    cases.push({ utf16:[65,random()%65536,32,random()%65536,66] });
  }
  cases.push({word:'a'.repeat(100000)}, {word:'A'.repeat(1048576)});
  const expected=[];
  for (const input of cases) expected.push((await actual(input)).output);
  const run=spawnSync('cargo',['run','--quiet','--offline','--locked','--manifest-path',path.join(root,'Cargo.toml'),'--example','oracle'],{
    input: cases.map(v=>JSON.stringify(v)).join('\n')+'\n', encoding:'utf8', maxBuffer:128*1024*1024,
  });
  if (run.error || run.status!==0) throw new Error(`Rust oracle process failed: ${run.error ?? run.stderr}`);
  const observed=run.stdout.trimEnd().split('\n').map(line=>JSON.parse(line));
  assert.equal(observed.length,cases.length,'No case/output truncation');
  for (let i=0;i<cases.length;i++) {
    const {word,...got}=observed[i];
    assert.deepEqual(got,expected[i],`case ${i}: ${JSON.stringify(cases[i]).slice(0,160)}`);
  }
  const failures={};
  for (const mode of ['copy-throw','copy-reject','toast-reject','open-reject']) failures[mode]=await actual({word:'APPLE'},mode);
  assert.deepEqual(failures['copy-throw'].events.map(v=>v[0]),['copy']);
  assert.deepEqual(failures['copy-reject'].events.map(v=>v[0]),['copy','toast','open']);
  assert.deepEqual(failures['toast-reject'].events.map(v=>v[0]),['copy','toast']);
  assert.equal(failures['open-reject'].threw,true);
  // Separate the deliberate native resource boundary from exact parity cases.
  const oversized={word:'a'.repeat(1048577)};
  const sourceOversized=await actual(oversized);
  assert.equal(sourceOversized.output.url.length,'https://mywordle.strivemath.com/?word='.length+1048577);
  const limited=spawnSync('cargo',['run','--quiet','--offline','--locked','--manifest-path',path.join(root,'Cargo.toml'),'--example','oracle'],{input:JSON.stringify(oversized)+'\n',encoding:'utf8',maxBuffer:8*1024*1024});
  if(limited.error||limited.status!==0)throw new Error(`Resource probe failed: ${limited.error??limited.stderr}`);
  assert.deepEqual(JSON.parse(limited.stdout),{validation:'Word exceeds the resource limit of 1048576 UTF-16 code units'});
  const report={cases:cases.length,compiler:ts.version,source:'5c7c4792f534a45001e7b71b1b41ead825914f80',failures,resourceBoundary:'source accepts 1048577; native explicitly rejects without truncation',observedAccepted:expected.filter(v=>v.url).length};
  fs.mkdirSync(path.join(root,'reports'),{recursive:true});fs.writeFileSync(path.join(root,'reports/oracle.json'),JSON.stringify(report,null,2)+'\n');
  console.log(`Source oracle: ${cases.length} exact parity cases; 4 recorded effect-failure cases; 1 explicit resource-boundary divergence; all individual UTF-16 units covered.`);
})().catch(e=>{console.error(e);process.exitCode=1;});
