#!/usr/bin/env python3
"""Explicit native cache experiment; never modifies a compiler checkout."""
import argparse, hashlib, json, os, pathlib, subprocess, time
p=argparse.ArgumentParser(); p.add_argument('--baseline', required=True); p.add_argument('--candidate', required=True); p.add_argument('--rustc', required=True); p.add_argument('--output',required=True); p.add_argument('--repo',required=True); a=p.parse_args()
root=pathlib.Path(a.output).resolve(); root.mkdir(parents=True,exist_ok=False)
records=[]
base_env={k:v for k,v in os.environ.items() if not k.startswith(('LUST_','LUSTR_'))}
def run(argv, cwd, env, expected=None):
    start=time.perf_counter_ns(); result=subprocess.run([str(x) for x in argv],cwd=cwd,env=env,capture_output=True,timeout=60)
    row={'argv':[str(x) for x in argv],'elapsed_ns':time.perf_counter_ns()-start,'exit':result.returncode,'stdout':result.stdout.decode(errors='replace'),'stderr':result.stderr.decode(errors='replace')}
    if expected is not None: row['expected_stdout']=expected
    records.append(row)
    with (root/'commands.jsonl').open('a') as trace: trace.write(json.dumps(row)+'\n')
    return row
cases=[]
for macro in ['env','option_env','include_str','include_bytes']:
    for label,gap in [('adjacent',''),('space',' '),('comment',' /* gap */ '),('nested',' /* a /* nested */ b */ '),('line',' // gap\n '),('long',' /* '+'nested '*60+' */ ')]:
        argument='"input.txt"' if macro.startswith('include') else '"LUSTR_CACHE_INPUT"'
        expression=f'{macro}{gap}!({argument})'
        source=f'fn main() {{ println!("{{:?}}", {expression}); }}\n'
        cases.append((f'{macro}-{label}',source,macro))
cases.append(('generated','macro_rules! call { ($name:ident) => { $name!("LUSTR_CACHE_INPUT") }; } fn main() { println!("{:?}", call!(option_env)); }\n','option_env'))
cases.append(('quote-line-counterexample','fn main(){ let _ = "env //"; println!("{:?}", option_env !("LUSTR_CACHE_INPUT")); }\n','option_env'))
cases.append(('quote-block-counterexample','fn main(){ let _ = "env /*"; println!("{:?}", option_env !("LUSTR_CACHE_INPUT")); let _ = "*/"; }\n','option_env'))
summary=[]
for label,source,macro in cases:
    cwd=root/label; cwd.mkdir(); f=cwd/'main.rs'; f.write_text(source)
    for value in [None,'first','second']:
        (cwd/'input.txt').write_text(value or 'unset')
        env=dict(base_env); env.pop('LUSTR_CACHE_INPUT',None)
        if value is not None: env['LUSTR_CACHE_INPUT']=value
        expected_success=not (macro=='env' and value is None)
        observations={}
        for lane,compiler in [('oracle',a.rustc),('baseline',a.baseline),('candidate',a.candidate)]:
            out=cwd/f'{lane}-app'; env_lane=dict(env)
            if lane=='oracle': args=[compiler,'--edition=2024','-Cpanic=unwind','-Coverflow-checks=on',f,'-o',out]
            else:
                env_lane.update(LUST_NO_DAEMON='1',LUST_NO_CACHE='0',LUST_CACHE_DIR=str(cwd/f'cache-{lane}'))
                args=[compiler,'build',f,'-o',out,'--timings']
            built=run(args,cwd,env_lane)
            built.update(case=label,lane=lane,input=value,source_sha256=hashlib.sha256(source.encode()).hexdigest())
            obs={'build_exit':built['exit'],'stdout':None,'cache_hit':'whole-request cache hit' in built['stderr']}
            if built['exit']==0:
                ran=run([out],cwd,env_lane); ran.update(case=label,lane=lane,input=value)
                obs.update(run_exit=ran['exit'],stdout=ran['stdout'])
            observations[lane]=obs
        oracle=observations['oracle']; baseline=observations['baseline']; candidate=observations['candidate']
        passed=(oracle['build_exit']==0)==expected_success and candidate['build_exit']==oracle['build_exit'] and candidate['stdout']==oracle['stdout'] and candidate.get('run_exit')==oracle.get('run_exit')
        summary.append({'case':label,'input':value,'oracle':oracle,'baseline':baseline,'candidate':candidate,'candidate_matches_oracle':passed,'baseline_matches_oracle':baseline['build_exit']==oracle['build_exit'] and baseline['stdout']==oracle['stdout']})
# These used to be v2's false-positive class; both lanes must retain request hits.
lookalikes={
 'variable-comment':'fn main(){ let env /* note */ = 7; println!("{}",env); }\n',
 'variable-space':'fn main(){ let env = 7; println!("{}",env); }\n',
 'variable-long-comment':'fn main(){ let env /* '+'comment '*60+' */ = 7; println!("{}",env); }\n',
 'literal-string':'fn main(){ let value = "env /* gap */ !"; println!("{}",value); }\n',
 'literal-raw-string':'fn main(){ let value = r#"env /* gap */ !"#; println!("{}",value); }\n',
}
for label,source in lookalikes.items():
    cwd=root/label; cwd.mkdir(); f=cwd/'main.rs'; f.write_text(source)
    for lane,compiler in [('baseline',a.baseline),('candidate',a.candidate)]:
        env=dict(base_env,LUST_NO_DAEMON='1',LUST_NO_CACHE='0',LUST_CACHE_DIR=str(cwd/f'cache-{lane}'))
        for step in range(2):
            built=run([compiler,'build',f,'-o',cwd/f'{lane}-app','--timings'],cwd,env)
            built.update(case=label,lane=lane,step=step)
            good=built['exit']==0 and (step==0 or 'whole-request cache hit' in built['stderr'])
            if built['exit']==0:
                ran=run([cwd/f'{lane}-app'],cwd,env); ran.update(case=label,lane=lane,step=step)
                good=good and ran['exit']==0
            summary.append({'case':label,'lane':lane,'step':step,'request_cache_behavior_preserved':good})
# Native panic recovery and destructor order must remain identical to rustc.
source=(pathlib.Path(a.repo)/'compiler/native/tests/unwind/unwind_drops.rs').read_text()
cwd=root/'panic-and-drop'; cwd.mkdir(); f=cwd/'main.rs'; f.write_text(source)
observations={}
for lane,compiler in [('oracle',a.rustc),('baseline',a.baseline),('candidate',a.candidate)]:
    env=dict(base_env,LUST_NO_DAEMON='1',LUST_NO_CACHE='0',LUST_CACHE_DIR=str(cwd/f'cache-{lane}'))
    out=cwd/f'{lane}-app'
    args=[compiler,'--edition=2024','-Cpanic=unwind',f,'-o',out] if lane=='oracle' else [compiler,'build',f,'-o',out]
    built=run(args,cwd,env); built.update(case='panic-and-drop',lane=lane)
    ran=run([out],cwd,env) if built['exit']==0 else None
    if ran: ran.update(case='panic-and-drop',lane=lane)
    observations[lane]={'build_exit':built['exit'],'run_exit':ran['exit'] if ran else None,'stdout':ran['stdout'] if ran else None}
expected={'build_exit':0,'run_exit':0,'stdout':'drop 2\ndrop 1\nresumed err=true dropped_sum=3\n'}
summary.append({'case':'panic-and-drop','candidate_matches_oracle':all(v==expected for v in observations.values()),'observations':observations})
(root/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
report={'schema':'lustr.cache-repair-correctness.v1','candidate_matches_all_oracle':all(s.get('candidate_matches_oracle',True) for s in summary),'lookalike_cache_preserved':all(s.get('request_cache_behavior_preserved',True) for s in summary),'baseline_failures':sum(s.get('baseline_matches_oracle',True)==False for s in summary),'commands':len(records),'cases':summary}
(root/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='cases'},indent=2))
raise SystemExit(0 if report['candidate_matches_all_oracle'] and report['lookalike_cache_preserved'] else 1)
