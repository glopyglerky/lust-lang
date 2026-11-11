#!/usr/bin/env python3
"""Isolated, unadmitted Lust cache experiment. Produces evidence only."""
import argparse,hashlib,json,os,pathlib,shutil,subprocess,time,traceback
BASE='ec89ca9342f50c77d2e1fd2ced2fc0c87829faab'
TREE='0ab1b09815802894e6078de0af59273cbdf46cb0'
p=argparse.ArgumentParser();p.add_argument('--work',required=True);a=p.parse_args()
repo=pathlib.Path(__file__).resolve().parents[3];tools=pathlib.Path(__file__).resolve().parent
work=pathlib.Path(a.work).resolve();work.mkdir(parents=True,exist_ok=False);evidence=work/'evidence';evidence.mkdir()
if work==repo or repo in work.parents:raise SystemExit('experimental work directory must be outside the primary checkout')
env={k:v for k,v in os.environ.items() if not k.startswith(('LUST_','LUSTR_')) and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']}
env.update(RUSTUP_TOOLCHAIN='1.99.1',CARGO_BUILD_JOBS='1')
deadline=time.monotonic()+3000;records=[];status={'admitted':False,'packaged':False,'base_revision':BASE,'upstream_tree':TREE}
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def capture(args,cwd=repo):return subprocess.check_output([str(x) for x in args],cwd=cwd,env=env,text=True).strip()
def execute(name,args,cwd=repo,timeout=900,localenv=None):
 remaining=deadline-time.monotonic()
 if remaining<=0:raise TimeoutError('experiment exceeded50minute internal deadline')
 start=time.monotonic();log=evidence/f'{name}.log'
 with log.open('w') as f:
  try:r=subprocess.run([str(x) for x in args],cwd=cwd,env=localenv or env,stdout=f,stderr=subprocess.STDOUT,timeout=min(timeout,remaining))
  except BaseException as error:
   records.append({'name':name,'argv':[str(x) for x in args],'cwd':str(cwd),'elapsed_seconds':time.monotonic()-start,'exception':repr(error),'log':log.name});raise
 records.append({'name':name,'argv':[str(x) for x in args],'cwd':str(cwd),'elapsed_seconds':time.monotonic()-start,'exit':r.returncode,'log':log.name})
 print(f'{name}: exit {r.returncode}',flush=True)
 (evidence/'steps.json').write_text(json.dumps(records,indent=2)+'\n')
 if r.returncode:raise RuntimeError(f'{name} failed with exit{r.returncode}; see {log.name}')
def compare_native_reports(before,after):
 for key in ['fixture_manifest_sha256','runner_sha256','configuration']:
  if before[key]!=after[key]:raise RuntimeError(f'native qualification binding differs: {key}')
 previous={case['name']:case for case in before['cases']};current={case['name']:case for case in after['cases']}
 if len(previous)!=len(before['cases']) or len(current)!=len(after['cases']) or previous.keys()!=current.keys():raise RuntimeError('native qualification case identities differ or duplicate')
 regressions=[]
 for name,old in previous.items():
  new=current[name]
  for key in ['category','source','source_sha256','kind']:
   if old[key]!=new[key]:raise RuntimeError(f'native qualification case binding differs: {name}/{key}')
  if old['lanes']['native']['passed'] and not new['lanes']['native']['passed']:regressions.append(name)
 return {'cases_compared':len(previous),'regressions':regressions,'passed':not regressions}
def candidate_identity(candidate):
 paths=capture(['git','diff','--name-only'],candidate).splitlines()
 if paths!=['compiler/native/src/driver/cache.rs']:raise RuntimeError(f'candidate has unexpected tracked edits: {paths}')
 if capture(['git','ls-files','--others','--exclude-standard','--','compiler/native'],candidate):raise RuntimeError('candidate has untracked native source additions')
 if sha(candidate/'compiler/native/src/driver/cache.rs')!=plan['candidate_cache_sha256']:raise RuntimeError('candidate source does not match preregistered patch')
 if capture(['git','rev-parse','HEAD:compiler/native'],candidate)!=TREE:raise RuntimeError('candidate parent tree mismatch')
 (evidence/'actual-candidate.patch').write_text(capture(['git','diff','--binary','--','compiler/native'],candidate)+'\n')
try:
 plan=json.loads((tools/'plan.json').read_text())
 for name,digest in plan['harness_sha256'].items():
  if sha(tools/name)!=digest:raise RuntimeError(f'preregistered input hash mismatch: {name}')
 if capture(['git','rev-parse',f'{BASE}:compiler/native'])!=TREE:raise RuntimeError('unexpected upstream baseline tree')
 status.update(primary_revision=capture(['git','rev-parse','HEAD']),rustc_version=capture(['rustc','-vV']),cargo_version=capture(['cargo','-vV']),input_hashes={p.name:sha(p) for p in tools.iterdir() if p.is_file()})
 shutil.copyfile(tools/'plan.json',evidence/'preregistered-plan.json');shutil.copyfile(tools/'candidate.patch',evidence/'proposed-candidate.patch')
 execute('build-source-guard',['rustc','--edition=2024','tools/lust-upstream-check.rs','-o',work/'lust-upstream-check'],timeout=60)
 execute('primary-source-before',[work/'lust-upstream-check'],timeout=60)
 baseline=work/'baseline';candidate=work/'candidate'
 execute('create-baseline',['git','worktree','add','--detach',baseline,BASE],timeout=120)
 execute('create-candidate',['git','worktree','add','--detach',candidate,BASE],timeout=120)
 execute('baseline-source-before',[work/'lust-upstream-check'],cwd=baseline,timeout=60)
 execute('apply-isolated-candidate',['git','apply','--check',tools/'candidate.patch'],cwd=candidate,timeout=30)
 execute('patch-isolated-candidate',['git','apply',tools/'candidate.patch'],cwd=candidate,timeout=30)
 candidate_identity(candidate)
 execute('build-shared-tinycc',['bash','compiler/native/scripts/build-tcc.sh'],cwd=baseline,timeout=600)
 env['LUST_TCC_HOME']=str(baseline/'compiler/native/third_party/tcc')
 tcc=pathlib.Path(env['LUST_TCC_HOME']);status['tinycc']={'revision':capture(['git','rev-parse','HEAD'],baseline/'compiler/native/third_party/tinycc'),'files':{str(p.relative_to(tcc)):sha(p) for p in sorted(tcc.rglob('*')) if p.is_file()}}
 for lane,checkout in [('baseline',baseline),('candidate',candidate)]:
  execute('build-'+lane,['cargo','build','--release','--locked','--manifest-path',checkout/'compiler/native/Cargo.toml','--target-dir',work/'build'/lane],cwd=checkout)
 # tcc_paths() reads these build-time env! literals when no package lib/lust
 # directory exists. Runtime LUST_TCC_HOME is deliberately not needed by probes.
 status['baked_tcc']={}
 for lane in ['baseline','candidate']:
  outputs=list((work/'build'/lane/'release/build').glob('lust-*/output'))
  if len(outputs)!=1:raise RuntimeError(f'{lane}: expected one native build-script output')
  text=outputs[0].read_text();shutil.copyfile(outputs[0],evidence/f'{lane}-build-script.log')
  settings=dict(line[len('cargo:rustc-env='):].split('=',1) for line in text.splitlines() if line.startswith('cargo:rustc-env='))
  expected={'LUST_TCC_LIB_PATH':str(tcc/'lib/tcc'),'LUST_TCC_BIN':str(tcc/'bin/tcc')}
  if any(settings.get(k)!=v for k,v in expected.items()):raise RuntimeError(f'{lane}: baked TinyCC paths differ from shared qualified installation')
  if (work/'build'/lane/'lib/lust').exists():raise RuntimeError(f'{lane}: unexpected runtime package override')
  status['baked_tcc'][lane]={k:settings[k] for k in [*expected,'LUST_BUILD_HASH']}
 status['compiler_sha256']={lane:{name:sha(work/'build'/lane/'release'/name) for name in ['lust','lust-core']} for lane in ['baseline','candidate']}
 status['native_sources']={'baseline_cache_sha256':sha(baseline/'compiler/native/src/driver/cache.rs'),'candidate_cache_sha256':sha(candidate/'compiler/native/src/driver/cache.rs'),'patch_sha256':sha(tools/'candidate.patch')}
 (evidence/'identities.json').write_text(json.dumps(status,indent=2)+'\n')
 execute('cache-unit-tests',['cargo','test','--locked','--profile','gate','--manifest-path',candidate/'compiler/native/Cargo.toml','--target-dir',work/'build/candidate-tests','--bin','lust-core','driver::cache::tests'],cwd=candidate,timeout=600)
 args=['--baseline',work/'build/baseline/release/lust','--candidate',work/'build/candidate/release/lust']
 rustc=capture(['rustup','which','--toolchain','1.99.1','rustc'])
 execute('external-input-correctness',['python3',tools/'correctness.py',*args,'--rustc',rustc,'--repo',repo,'--output',evidence/'correctness'],timeout=600)
 execute('daemon-external-input-correctness',['python3',tools/'daemon.py',*args,'--output',evidence/'daemon'],timeout=300)
 for lane in ['baseline','candidate']:
  execute('native-coverage-'+lane,['python3',repo/'tools/native-qualification.py','--lust',work/'build'/lane/'release/lust','--rustc',rustc,'--output',evidence/('native-'+lane)],timeout=300)
 coverage=compare_native_reports(json.loads((evidence/'native-baseline/report.json').read_text()),json.loads((evidence/'native-candidate/report.json').read_text()))
 (evidence/'native-coverage-comparison.json').write_text(json.dumps(coverage,indent=2)+'\n')
 if not coverage['passed']:raise RuntimeError(f'native candidate loses baseline qualification: {coverage["regressions"]}')
 execute('paired-compiler-benchmark',['python3',tools/'benchmark.py',*args,'--rustc',rustc,'--repo',repo,'--output',evidence/'performance','--rounds','40'],timeout=600)
 execute('baseline-source-after',[work/'lust-upstream-check'],cwd=baseline,timeout=60)
 candidate_identity(candidate)
 report=json.loads((evidence/'performance/report.json').read_text());status['performance_decision']={k:report[k] for k in ['equal_case_weight_geomean','stratified_bootstrap95_geomean','faster_rule_met','confirmed_over5percent_regressions']}
 status['candidate_passes_experimental_speed_rule']=report['faster_rule_met'] and not report['confirmed_over5percent_regressions']
 status['experiment_completed']=True
except BaseException as error:
 status.update(experiment_completed=False,error=repr(error),traceback=traceback.format_exc())
 print(status['traceback'],flush=True)
finally:
 try:execute('primary-source-after',[work/'lust-upstream-check'],timeout=60)
 except BaseException as error:status['primary_source_check_error']=repr(error);status['experiment_completed']=False
 status.update(admitted=False,packaged=False)
 (evidence/'steps.json').write_text(json.dumps(records,indent=2)+'\n')
 (evidence/'result.json').write_text(json.dumps(status,indent=2)+'\n')
print(json.dumps(status,indent=2))
raise SystemExit(0 if status.get('experiment_completed') else 1)
