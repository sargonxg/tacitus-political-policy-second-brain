#!/usr/bin/env python3
"""Deterministic evidence-retention comparison. No inference or answer-quality score."""
import argparse, hashlib, json, pathlib, re, subprocess, tempfile, time
STOP={'the','and','what','does','about','with','this','that','for','are'}
def tokens(s): return {x.lower() for x in re.findall(r'[^\W_]+',s,flags=re.UNICODE) if len(x)>2 and x.lower() not in STOP}
def encoded(v): return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
def baseline(c,q,chunked=False):
    docs=c['documents']; now=q['known_as_of']; valid=q['valid_at']
    if not 2048<=q['max_bytes']<=250000 or not q['question'].strip(): raise ValueError('bounded query required')
    if q['revoked_sources']: raise ValueError('atomic revocation refusal')
    if now<c['graph']['available_at']: raise ValueError('snapshot unavailable')
    if c['manifest']['valid_until'] is not None and valid>=c['manifest']['valid_until']: raise ValueError('manifest expired')
    for d in docs:
        key=f"{d['id']}@{d['version']}"
        if d['workspace']!=c['manifest']['scope'] or d['withdrawn'] or d['restricted'] or not d['redistributable'] or not d['license'] or key in q['revoked_sources'] or d['available_at']>now or (d['published_at'] is not None and d['published_at']>now) or (d['valid_from'] is not None and d['valid_from']>valid) or (d['valid_until'] is not None and valid>=d['valid_until']): raise ValueError('source ineligible')
    out={'format':'evaluation-whole-document-lexical/1','question':q['question'],'known_as_of':now,'valid_at':valid,'manifest':c['manifest'],'trust':'untrusted source data; unknown validity is not proof; no generated answer','documents':[],'omitted_sources':[]}
    units=[]
    for d in docs:
        if chunked:
            offset=0
            for line in d['text'].splitlines(keepends=True):
                end=offset+len(line.encode())
                if line.strip(): units.append((d,offset,end,line))
                offset=end
        else: units.append((d,0,len(d['text'].encode()),d['text']))
    units.sort(key=lambda u:(-len(tokens(q['question'])&tokens(u[3])),u[0]['id'],u[1]))
    out['chunks']=[]
    for d,start,end,quote in units:
        score=len(tokens(q['question'])&tokens(quote))
        if score and q['time_mode']=='require_declared_interval' and (d['valid_from'] is None or d['valid_until'] is None): raise ValueError('unknown validity')
        trial=json.loads(json.dumps(out));key=f"{d['id']}@{d['version']}"
        if chunked:
            if not any(x['id']==d['id'] and x['version']==d['version'] for x in trial['documents']):
                meta=dict(d);meta['text']='';trial['documents'].append(meta)
            trial['chunks'].append({'source':key,'start':start,'end':end,'quote':quote,'text_hash':d['text_hash']})
        else: trial['documents'].append(d)
        included={f"{x['id']}@{x['version']}" for x in trial['documents']}
        trial['omitted_sources']=[f"{x['id']}@{x['version']}" for x in docs if f"{x['id']}@{x['version']}" not in included]
        if score and len(encoded(trial))<=q['max_bytes']: out=trial
    included={f"{x['id']}@{x['version']}" for x in out['documents']}
    out['omitted_sources']=[f"{x['id']}@{x['version']}" for x in docs if f"{x['id']}@{x['version']}" not in included]
    if len(encoded(out))>q['max_bytes']: raise ValueError('budget cannot hold receipt')
    return encoded(out)
def coverage(c,payload,method):
    obj=json.loads(payload); covered=[]
    originals={f"{s['id']}@{s['version']}":s for s in c['documents']}
    def source_slice(e): return originals[e['source']]['text'].encode()[e['start']:e['end']].decode()
    for a in c['assertions']:
        if method=='capsule':
            claim=obj['context']['claims'].get(a['id'])
            ok=bool(claim) and claim['assertion']==a and all(any(q['span']==e and q['text']==source_slice(e) for q in claim['citations']) for e in a['evidence'])
        else:
            available={f"{s['id']}@{s['version']}":s for s in obj['documents']}
            def retained(e):
                d=available.get(e['source'])
                if not d or d['text_hash']!=e['text_hash']:return False
                b=d['text'].encode()
                if hashlib.sha256(b).hexdigest()==e['text_hash'] and b[e['start']:e['end']].decode()==source_slice(e):return True
                original=originals[e['source']]['text'].encode()
                return any(ch['source']==e['source'] and ch['text_hash']==e['text_hash'] and ch['start']<=e['start'] and ch['end']>=e['end'] and ch['quote'].encode()==original[ch['start']:ch['end']] for ch in obj.get('chunks',[]))
            ok=all(retained(e) for e in a['evidence'])
        if ok: covered.append(a['id'])
    return covered

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--fixtures',required=True);ap.add_argument('--output',required=True);a=ap.parse_args()
    out=pathlib.Path(a.output)
    if out.exists() or out.is_symlink(): raise SystemExit('output must be a fresh file')
    root=pathlib.Path(a.fixtures);cases=[]
    with tempfile.TemporaryDirectory() as td:
        for name,queries in [('knowledge',[('withdraws',['access','withdrawal']),('heat exercise',['heat']),('astronomy galaxies',[])]),('minister',[('discours',['application-en','application-fr','phased-exceptions','speech-guidance','version-warning']),('applicable',['application-fr']),('publication institutions',['publication-record'])])]:
            c=json.loads((root/name/'capture.json').read_text()); archive=pathlib.Path(td)/(name+'.zip')
            subprocess.run([a.binary,'pack',str(root/name/'capture.json'),str(archive)],check=True,capture_output=True)
            for question,expected in queries:
                for budget in [8192,16384,32768]:
                    q={'question':question,'known_as_of':c['manifest']['created_at'],'valid_at':c['manifest']['created_at'],'time_mode':'disclose_unknown','max_bytes':budget,'revoked_sources':[]}
                    row={'corpus':name,'question':question,'expected_support':expected,'max_bytes':budget,'methods':{}}
                    for method in ['whole_document','line_chunk','capsule']:
                        samples=[];digest=None;record=None
                        for _ in range(3):
                            start=time.perf_counter_ns()
                            if method=='capsule':
                                r=subprocess.run([a.binary,'reuse',str(archive)],input=encoded(q),capture_output=True);payload=r.stdout;error=r.returncode!=0
                            else:
                                try:payload=baseline(c,q,chunked=method=='line_chunk');error=False
                                except ValueError:payload=b'';error=True
                            samples.append(time.perf_counter_ns()-start)
                            h=hashlib.sha256(payload).hexdigest()
                            if digest is not None: assert digest==h
                            digest=h
                            covered=[] if error else coverage(c,payload,method)
                            assert len(payload)<=budget
                            record={'error':error,'bytes':len(payload),'covered_claim_evidence':covered,'required_support_complete':bool(expected) and set(expected)<=set(covered),'payload_sha256':h}
                        record['elapsed_ns_samples']=samples;row['methods'][method]=record
                    cases.append(row)
    report={'scope':'deterministic structural retention, not factual/answer quality or model evaluation','cases':cases,'controls':['same exact source revisions and questions','equal actual serialized byte limits','same atomic source eligibility and snapshot cutoff policy','no generator or model calls'], 'limitations':['Hand-authored graph annotations and research notes add information and labor; not a free semantic improvement.','Source span coverage is not semantic support or correct answer generation.','Whole-document baseline retains original text but not separately authored assertion conditions; capsule preserves structured interpretation.','Timing is descriptive and not comparable: Rust subprocess startup versus in-process Python baseline.','Six illustrative queries are not held-out or representative; 35-case blueprint is not fully executed.']}
    with out.open('x') as f: json.dump(report,f,ensure_ascii=False,indent=2)
if __name__=='__main__':main()
