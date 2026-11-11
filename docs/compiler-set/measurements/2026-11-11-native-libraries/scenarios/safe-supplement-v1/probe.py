from pathlib import Path
import json,os,subprocess,shutil,sys,importlib.util,hashlib
here=Path(__file__).resolve().parent
fixture=Path('/workspace/scratch/0fa08655b84f/lustr-library-tests-work/tests/native-library')
runner=Path('/workspace/scratch/0fa08655b84f/lustr-library-tests-work/tools/qualify-native-library.py')
spec=importlib.util.spec_from_file_location('gate',runner);gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)
native=Path('/workspace/scratch/0fa08655b84f/lustr-library-correctness/gate/lustr-native')
oracle=Path('/workspace/scratch/0fa08655b84f/build-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc')
cargo=oracle.with_name('cargo')
report={'scope':'Supplemental safe primitive libraries only. Full original fixture still fails unsafe producer parsing. This diagnostic uses separate safe producer input and does not replace that required gate.','qualified':False,'no_trace_qualification':True,'full_required_suite_passed':False,'native_sha256':gate.digest(native),'commands':[],'checks':[]}
projects={};envs={};calls={};archives={};hidden=[]
def run(label,lane,args,cwd=None):
    r=subprocess.run([str(a) for a in args],cwd=cwd or projects[lane],env=envs[lane],capture_output=True,text=True,timeout=120)
    row={'label':label,'lane':lane,'argv':[str(a) for a in args],'cwd':str(cwd or projects[lane]),'exit_code':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
    report['commands'].append(row)
    (here/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    return row
def check(c,m):
    if not c:raise AssertionError(m)
def replay(lane,case,expect=0,stdout=None,stderr_parts=()):
    call=calls[lane];out=here/(lane+' replay '+case);out.mkdir()
    argv=call['argv'].copy();argv[next(i for i,s in enumerate(argv) if s.endswith('.rs'))]=str(projects[lane]/'cases'/f'{case}.rs');argv[argv.index('--out-dir')+1]=str(out)
    saved=envs[lane];envs[lane]={**saved,**call['environment']}
    try:row=run(case,lane,argv,projects[lane]/'app')
    finally:envs[lane]=saved
    check(row['exit_code']==expect,row)
    if expect==0:
        bins=[p for p in out.iterdir() if p.is_file() and os.access(p,os.X_OK)];check(len(bins)==1,bins)
        r=run(case+'-runtime',lane,bins)
        target_exit=101 if case=='imported_overflow' else 0
        check(r['exit_code']==target_exit and r['stdout']==stdout,r)
        check(all(s in r['stderr'] for s in stderr_parts),r)
try:
    for lane,compiler in [('oracle',oracle),('native',native)]:
        project=here/(lane+' safe project');shutil.copytree(fixture,project);projects[lane]=project
        left=project/'left/src/lib.rs';s=left.read_text();block='pub unsafe fn unchecked_add(value: i32) -> i32 {\n    value + 17\n}\n\n';check(block in s,'missing unsafe block');left.write_text(s.replace(block,''))
        env={k:v for k,v in os.environ.items() if not k.startswith(('CARGO_','RUST','LUST_','LUSTR_','LD_','DYLD_'))};env.update(PATH='/usr/bin:/bin',CARGO_HOME=str(here/(lane+' cargo home')),RUSTUP_HOME=str(here/(lane+' rustup home')),CARGO_NET_OFFLINE='true',RUSTC=str(compiler),RUSTC_WRAPPER='',RUSTC_WORKSPACE_WRAPPER='');envs[lane]=env
        row=run('cargo-initial',lane,[cargo,'build','--offline','--locked','-vv','--message-format=json','--target-dir',here/(lane+' target'),'-p','lustr_library_contract']);check(row['exit_code']==0,row)
        inv=gate.compiler_invocations(row['stderr'],compiler);calls[lane]=next(c for c in inv if gate.option(c['argv'],'--crate-name')=='lustr_library_contract');archives[lane]={s.split('=',1)[0]:Path(s.split('=',1)[1]) for s in calls[lane]['argv'] if s.startswith(('left=','right='))}
        row=run('runtime',lane,[here/(lane+' target')/'debug/lustr_library_contract']);check(row['exit_code']==0 and row['stdout']=='left=31 right=109 local=21\n',row)
    report['checks'].append('real Cargo safe two same-named versions produce correct runtime')
    report['metadata']={}
    for alias,path in archives['native'].items():
        members=gate.read_ar(path);meta=next(b for n,b in members if n=='lustr.meta');objects=[(n,b) for n,b in members if n.endswith('.o')];report['metadata'][alias]=gate.inspect_metadata(meta,objects)
    for lane in ('oracle','native'):
        for dep in ('left','right'):
            old=projects[lane]/dep/'src';new=projects[lane]/dep/'hidden-source';old.rename(new);hidden.append((old,new))
        for case,stdout in [('primitive_abi','primitive ABI passed\n'),('conditional_consumer','conditional=31\n')]:replay(lane,case,stdout=stdout)
        for case in ('private','type_mismatch','arity_mismatch','result_mismatch'):replay(lane,case,expect=1)
        replay(lane,'imported_overflow',stdout='',stderr_parts=('attempt to add with overflow','left/src/lib.rs:2'))
    report['checks'].append('source-hidden scalar ABI, function pointers, cfg, semantic rejection and imported panic location')
    for case in ('public_generic','public_type','public_const','public_macro','public_must_use','public_deprecated','public_no_mangle'):
        for lane,compiler in [('oracle',oracle),('native',native)]:
            out=here/(lane+'-'+case+'.rlib');row=run(case,lane,[compiler,'--crate-name','unsupported','--edition=2024',projects[lane]/'cases'/f'{case}.rs','--crate-type','rlib','--emit=link','-o',out])
            check((row['exit_code']==0 and out.exists()) if lane=='oracle' else (row['exit_code'] in (1,2) and not out.exists()),row)
    report['safe_supplement_success']=True
except BaseException as e:
    report['failure']=str(e);raise
finally:
    for old,new in reversed(hidden):
        if new.exists() and not old.exists():new.rename(old)
    report['fixture_sha256']={p.relative_to(here).as_posix():gate.digest(p) for project in projects.values() for p in project.rglob('*') if p.is_file() and p.suffix in ('.rs','.toml')}
    (here/'report.json').write_text(json.dumps(report,indent=2)+'\n')
