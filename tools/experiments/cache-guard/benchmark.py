#!/usr/bin/env python3
"""Paired actual-compiler experiment, with fixed samples and runtime checks."""
import argparse,hashlib,json,math,os,pathlib,platform,random,statistics,subprocess,time
p=argparse.ArgumentParser(); p.add_argument('--baseline',required=True); p.add_argument('--candidate',required=True); p.add_argument('--rustc',required=True); p.add_argument('--repo',required=True); p.add_argument('--output',required=True); p.add_argument('--rounds',type=int,default=40); a=p.parse_args()
root=pathlib.Path(a.output).resolve(); root.mkdir(parents=True,exist_ok=False)
repo=pathlib.Path(a.repo).resolve(); records=[]; samples=[]
base={k:v for k,v in os.environ.items() if not k.startswith(('LUST_','LUSTR_'))}
base.update(LUST_NO_DAEMON='1',LUST_NO_CACHE='0',LUST_THREADS='1',LUSTR_CACHE_INPUT='unchanged')
affinity=sorted(os.sched_getaffinity(0)); selected_cpu=affinity[0]; os.sched_setaffinity(0,{selected_cpu})
compilers={'baseline':pathlib.Path(a.baseline).resolve(),'candidate':pathlib.Path(a.candidate).resolve()}
existing=(repo/'tests/native-qualification/existing-generics-traits.rs').read_text()
logscan=(repo/'compiler/native/bench/real/main.rs').read_text()
cases=[
 ('fresh-existing','fresh',existing,None),
 ('request-existing','request',existing,None),
 ('body-edit','body',existing,None),
 ('signature-edit','signature','fn value(x: i32) -> i32 { x + 1 } fn main(){ println!("{}",value(40)); }\n',None),
 ('invalid-edit','invalid-edit','fn main(){ let value: bool = true; println!("{}",value); }\n',None),
 ('invalid-request','invalid-request','fn main(){ let value: bool = 7; println!("{}",value); }\n',None),
 ('early-option-env','request','fn main(){ println!("{:?}",option_env!("LUSTR_CACHE_INPUT")); }\n',None),
 ('variable-space','request','fn main(){ let env = 7; println!("{}",env); }\n',None),
 ('variable-comment','request','fn main(){ let env /* note */ = 7; println!("{}",env); }\n',None),
 ('variable-long-comment','request','fn main(){ let env /* '+'comment '*60+' */ = 7; println!("{}",env); }\n',None),
 ('literal-string','request','fn main(){ let value = "env /* note */ !"; println!("{}",value); }\n',None),
 ('fresh-logscan','fresh',logscan,None),
 ('request-logscan','request',logscan,None),
 ('generated-option-env','request','macro_rules! call { ($name:ident) => { $name!("LUSTR_CACHE_INPUT") }; } fn main(){ println!("{:?}",call!(option_env)); }\n',None),
]
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def execute(argv,cwd,env,tag):
    start=time.perf_counter_ns(); r=subprocess.run([str(v) for v in argv],cwd=cwd,env=env,capture_output=True,timeout=90); elapsed=time.perf_counter_ns()-start
    row={'tag':tag,'argv':[str(v) for v in argv],'elapsed_ns':elapsed,'exit':r.returncode,'stdout':r.stdout.decode(errors='replace'),'stderr':r.stderr.decode(errors='replace')}
    records.append(row)
    with (root/'commands.jsonl').open('a') as trace: trace.write(json.dumps(row)+'\n')
    return row
oracle_outputs={}
def oracle(label,source,expected_exit=0):
    key=hashlib.sha256(source.encode()).hexdigest()
    if key in oracle_outputs:return oracle_outputs[key]
    cwd=root/'oracle'/label; cwd.mkdir(parents=True,exist_ok=True); src=cwd/'main.rs'; src.write_text(source)
    b=execute([a.rustc,'--edition=2024','-Cpanic=unwind','-Coverflow-checks=on',src,'-o',cwd/'app'],cwd,base,'oracle-build')
    if b['exit']!=expected_exit:raise RuntimeError(f'oracle verdict {label}: {b}')
    if expected_exit: out=None
    else:
        r=execute([cwd/'app'],cwd,base,'oracle-run')
        if r['exit']!=0:raise RuntimeError(f'oracle runtime {label}: {r}')
        out=r['stdout']
    oracle_outputs[key]=out; return out
def build(lane,cwd,cache,source,expected_exit,expected_stdout,tag):
    src=cwd/'main.rs'; src.write_text(source); env=dict(base,LUST_CACHE_DIR=str(cache)); out=cwd/f'app-{lane}'
    b=execute([compilers[lane],'build',src,'-o',out,'--timings'],cwd,env,tag)
    if b['exit']!=expected_exit:raise RuntimeError(f'native verdict {lane} {tag}: {b}')
    if expected_exit:
        if 'error[L0110]' not in b['stderr']:raise RuntimeError(f'unexpected rejection {tag}: {b}')
    else:
        r=execute([out],cwd,env,'runtime-'+tag)
        if r['exit']!=0 or r['stdout']!=expected_stdout:raise RuntimeError(f'native runtime {lane} {tag}: {r}')
    return b
# Resolve every fixture's semantic result before measuring any samples.
for label,mode,source,_ in cases:
    oracle(label,source,1 if mode=='invalid-request' else 0)
    if mode=='body':
        for round in range(a.rounds): oracle(f'{label}-{round}',source.replace('Value(3)',f'Value({round+5})'))
    if mode=='signature':oracle(label+'-edit',source.replace('i32','i64'))
    if mode=='invalid-edit':oracle(label+'-edit',source.replace('true','7'),1)
for label,mode,source,_ in cases:
    if mode in ['request','invalid-request']:
        cwd=root/'work'/label; cwd.mkdir(parents=True)
        for lane in compilers:build(lane,cwd,cwd/f'cache-{lane}',source,1 if mode=='invalid-request' else 0,oracle_outputs[hashlib.sha256(source.encode()).hexdigest()],'warmup-'+label)
for round in range(a.rounds):
    # Rotate the scenario start every round, while balancing lane order within each case.
    order=cases[round%len(cases):]+cases[:round%len(cases)]
    for label,mode,source,_ in order:
        cwd=root/'work'/label; cwd.mkdir(parents=True,exist_ok=True)
        edited=source; expected_exit=1 if mode=='invalid-request' else 0
        if mode=='body': edited=source.replace('Value(3)',f'Value({round+5})')
        if mode=='signature': edited=source.replace('i32','i64')
        if mode=='invalid-edit':edited=source.replace('true','7'); expected_exit=1
        expected=oracle_outputs[hashlib.sha256(edited.encode()).hexdigest()]
        lanes=['baseline','candidate'] if round%2==0 else ['candidate','baseline']
        if mode in ['body','signature','invalid-edit']:
            for lane in lanes:build(lane,cwd,cwd/f'round-{round}-{lane}',source,0,oracle_outputs[hashlib.sha256(source.encode()).hexdigest()],'seed-'+label)
        for lane in lanes:
            cache=cwd/(f'round-{round}-{lane}' if mode in ['fresh','body','signature','invalid-edit'] else f'cache-{lane}')
            row=build(lane,cwd,cache,edited,expected_exit,expected,f'measured-{label}-{round}-{lane}')
            samples.append({'case':label,'round':round,'lane':lane,'first_in_pair':lane==lanes[0],'elapsed_ns':row['elapsed_ns'],'source_sha256':hashlib.sha256(edited.encode()).hexdigest(),'cache_hit':'whole-request cache hit' in row['stderr'],'exit':row['exit']})
    (root/'samples.json').write_text(json.dumps(samples,indent=2)+'\n')
    print(f'completed round {round+1}/{a.rounds}',flush=True)
(root/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
rng=random.Random(20261111)
case_stats=[]; all_logs=[]; per_case_logs=[]
for label,*_ in cases:
    subset=[s for s in samples if s['case']==label]; pairs=[]
    for round in range(a.rounds):
        pair={s['lane']:s for s in subset if s['round']==round}; pairs.append(pair['candidate']['elapsed_ns']/pair['baseline']['elapsed_ns'])
    logs=[math.log(r) for r in pairs]; all_logs+=logs; per_case_logs.append(logs)
    boots=sorted(math.exp(statistics.mean(rng.choices(logs,k=len(logs)))) for _ in range(10000))
    case_stats.append({'case':label,'pairs':len(pairs),'baseline_median_ms':statistics.median(s['elapsed_ns']/1e6 for s in subset if s['lane']=='baseline'),'candidate_median_ms':statistics.median(s['elapsed_ns']/1e6 for s in subset if s['lane']=='candidate'),'median_paired_ratio':statistics.median(pairs),'geometric_mean_ratio':math.exp(statistics.mean(logs)),'bootstrap95_geomean':[boots[250],boots[9749]],'baseline_request_hits':sum(s['cache_hit'] for s in subset if s['lane']=='baseline'),'candidate_request_hits':sum(s['cache_hit'] for s in subset if s['lane']=='candidate')})
boots=sorted(math.exp(statistics.mean(statistics.mean(rng.choices(logs,k=len(logs))) for logs in per_case_logs)) for _ in range(10000))
summary={'schema':'lustr.cache-guard-paired-compiler.v1','rounds':a.rounds,'sample_count':len(samples),'host':platform.platform(),'cpu_affinity_before':affinity,'cpu_affinity_used':[selected_cpu],'build_profile':'release opt3 cgu1 ThinLTO Rust1.99.1','compiler_hashes':{lane:{name:sha(path.parent/name) for name in ['lust','lust-core']} for lane,path in compilers.items()},'environment':{k:v for k,v in base.items() if k.startswith(('LUST_','LUSTR_'))},'cases':case_stats,'equal_case_weight_geomean':math.exp(statistics.mean(all_logs)),'stratified_bootstrap95_geomean':[boots[250],boots[9749]],'faster_rule_met':boots[9749]<1,'confirmed_over5percent_regressions':[s['case'] for s in case_stats if s['bootstrap95_geomean'][0]>1.05],'all_compilation_and_runtime_checks_passed':True,'scope':'These fixtures and host only; paired observations assume temporal stationarity within alternating pairs. No universal claim.'}
(root/'report.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
