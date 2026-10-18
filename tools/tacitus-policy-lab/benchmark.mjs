// SPDX-License-Identifier: Apache-2.0
import {readFile, writeFile} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import {cpus} from 'node:os';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {sha256} from './core.mjs';
import {benchmarkInputs, CHANGE_INPUT} from './benchmark-fixtures.mjs';
import {ARMS, prepareCase, selectContext} from './benchmark-selection.mjs';
import {scoreContext, benchmarkChange} from './benchmark-metrics.mjs';

const WARMUPS = 3;
export function parseBenchmarkArgs(args) {
  const options = {iterations:25, out:null};
  const seen = new Set();
  for (let i=0; i<args.length; i++) {
    const flag = args[i];
    if (!['--iterations', '--out'].includes(flag) || seen.has(flag)) throw new Error('Unknown or repeated benchmark option');
    seen.add(flag);
    const value = args[++i];
    if (!value || value.startsWith('--')) throw new Error(`${flag} requires a value`);
    if (flag === '--out') options.out = value;
    else {
      if (!/^[1-9][0-9]*$/.test(value) || Number(value)>1000) throw new Error('--iterations must be an integer from 1 to 1000');
      options.iterations = Number(value);
    }
  }
  return options;
}
export function timingSummary(samples) {
  const ordered = [...samples].sort((a,b) => a-b);
  const percentile = q => ordered[Math.max(0, Math.ceil(q*ordered.length)-1)];
  return {samples:ordered.length, min_ms:ordered[0], p50_ms:percentile(0.50), p95_ms:percentile(0.95), max_ms:ordered.at(-1)};
}
async function measure(fn) {
  const start = performance.now();
  const result = await fn();
  return {result, ms:performance.now()-start};
}

export async function runBenchmark({iterations=25}={}) {
  if (!Number.isInteger(iterations) || iterations<1 || iterations>1000) throw new Error('Invalid benchmark iteration count');
  const gold = JSON.parse(await readFile(new URL('./benchmark-gold.json', import.meta.url), 'utf8'));
  const fingerprints = {};
  for (const file of ['core.mjs','ontology.mjs','demo.mjs','benchmark-fixtures.mjs','benchmark-gold.json',
    'benchmark-selection.mjs','benchmark-metrics.mjs','benchmark.mjs','benchmark.test.mjs','cli.mjs','package.json','change-review.mjs','change-review.test.mjs']) {
    fingerprints[file] = await sha256(await readFile(new URL(file, import.meta.url)));
  }
  const cases = [];
  for (const input of await benchmarkInputs()) {
    const preparation = await measure(() => prepareCase(input));
    const prepared = preparation.result;
    const outputs = new Map(), samples = new Map(ARMS.map(arm => [arm, []]));
    const rows = [];
    for (const arm of ARMS) {
      const first = await measure(() => selectContext(prepared, arm));
      outputs.set(arm, first.result);
      rows.push({arm, max_bytes:first.result.packet.limits.max_bytes, packet_bytes:first.result.receipt.bytes,
        receipt_digest:first.result.receipt.digest, omissions:first.result.receipt.omissions,
        scores:await scoreContext(first.result, input, gold.cases[input.id]),
        ...(first.result.change_review ? {change_review:first.result.change_review} : {}), timing:{first_observed_ms:first.ms}});
    }
    for (let iteration=0; iteration<WARMUPS+iterations; iteration++) {
      // Rotate arm order to avoid always measuring the same comparator last.
      for (let offset=0; offset<ARMS.length; offset++) {
        const arm = ARMS[(iteration+offset)%ARMS.length];
        const measured = await measure(() => selectContext(prepared, arm));
        if (measured.result.receipt.digest !== outputs.get(arm).receipt.digest) throw new Error('Nondeterministic context receipt');
        if (iteration>=WARMUPS) samples.get(arm).push(measured.ms);
      }
    }
    for (const row of rows) Object.assign(row.timing, timingSummary(samples.get(row.arm)));
    cases.push({case_id:input.id, known_as_of:input.contract.known_as_of, valid_at:input.contract.valid_at,
      review_since:input.review_since, eligible_ids:prepared.assertions.map(a => a.id), eligibility_omissions:prepared.eligibility_omissions,
      eligibility_ms:preparation.ms, arms:rows});
  }
  return {
    format:'tacitus-policy-lab-benchmark/1', status:'synthetic-developer-regression-only',
    environment:{node:process.version, platform:process.platform, arch:process.arch, cpu:cpus()[0]?.model ?? 'unknown'},
    method:{cases:4, dossiers:2, warmups:WARMUPS, measured_iterations:iterations,
      timing:'In-process context compilation only, including hashing/serialization. Eligibility measured separately; fixture construction, scoring, I/O, model and human work excluded. First observed is not cold start.',
      reference:'all-eligible-reference has a 250000-byte limit verified to fit these fixtures; not equal-budget and not a full-document/model baseline.',
      candidate:'change-first uses an explicit previous knowledge checkpoint, not semantic correction detection. All bounded arms see the same legitimate evidence and budget; new records can displace older critical material.',
      quality:'Developer-authored labels, same fixtures used for development; not held-out or independently reviewed.',
      effort:'Reviewable-record counts only; dependency creation/maintenance and analyst time not measured.'},
    fingerprints, cases, change_impact:benchmarkChange(CHANGE_INPUT, gold.change),
    unmeasured:{human_quality:null, analyst_minutes:null, model_quality:null, model_latency:null, production_performance:null},
    network_calls:0, model_calls:0, canonical_writes:0
  };
}

export async function benchmarkMain(args) {
  const {iterations, out} = parseBenchmarkArgs(args);
  const report = JSON.stringify(await runBenchmark({iterations}), null, 2)+'\n';
  if (out) {
    await writeFile(resolve(out), report, {flag:'wx', mode:0o600});
    console.log(`Created synthetic benchmark report: ${resolve(out)}`);
  } else process.stdout.write(report);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  benchmarkMain(process.argv.slice(2)).catch(error => { console.error(error.message); process.exitCode=1; });
}
