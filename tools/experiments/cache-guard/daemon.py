#!/usr/bin/env python3
"""Bounded warm-session external-input checks, with owned daemon cleanup."""
import argparse,json,os,pathlib,subprocess,tempfile,time
p=argparse.ArgumentParser();p.add_argument('--baseline',required=True);p.add_argument('--candidate',required=True);p.add_argument('--output',required=True);a=p.parse_args()
root=pathlib.Path(a.output).resolve();root.mkdir(parents=True,exist_ok=False);records=[];results=[]
base={k:v for k,v in os.environ.items() if not k.startswith(('LUST_','LUSTR_'))}
def run(argv,cwd,env):
 r=subprocess.run([str(x) for x in argv],cwd=cwd,env=env,capture_output=True,text=True,timeout=60)
 row={'argv':[str(x) for x in argv],'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr};records.append(row);return row
try:
 with tempfile.TemporaryDirectory(prefix='lustr-cache-daemon-') as tmp:
  for lane,compiler in [('baseline',a.baseline),('candidate',a.candidate)]:
   for macro in ['env','option_env','include_str']:
    cwd=root/f'{lane}-{macro}';cwd.mkdir();socket=pathlib.Path(tmp)/f'{lane}-{macro}.sock';cache=cwd/'cache'
    env=dict(base,LUST_NO_DAEMON='0',LUST_DAEMON_SOCKET=str(socket),LUST_DAEMON_IDLE='120',LUST_CACHE_DIR=str(cache),LUST_NO_CACHE='0')
    env.pop('LUSTR_CACHE_INPUT',None);source=cwd/'main.rs';argument='"input.txt"' if macro=='include_str' else '"LUSTR_CACHE_INPUT"'
    source.write_text(f'fn main() {{ println!("{{:?}}", {macro} /* gap */ ! ({argument})); }}\n')
    core=pathlib.Path(compiler).resolve().with_name('lust-core')
    with (cwd/'daemon.log').open('w') as log:
     daemon=subprocess.Popen([str(core),'daemon'],cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT)
     try:
      deadline=time.monotonic()+15
      while not socket.exists():
       if daemon.poll() is not None or time.monotonic()>deadline:raise RuntimeError('daemon failed to bind its owned socket')
       time.sleep(.03)
      for value in [None,'first','second']:
       current=dict(env);current.pop('LUSTR_CACHE_INPUT',None)
       if value is not None:current['LUSTR_CACHE_INPUT']=value
       (cwd/'input.txt').write_text(value or 'unset')
       if daemon.poll() is not None:raise RuntimeError('daemon exited before request')
       out=cwd/'app';built=run([compiler,'build',source,'-o',out,'--timings'],cwd,current)
       expected_exit=1 if macro=='env' and value is None else 0
       if macro=='option_env':expected='None\n' if value is None else f'Some("{value}")\n'
       else:expected=f'"{value or "unset"}"\n'
       ran=run([out],cwd,current) if built['exit']==0 else None
       good=built['exit']==expected_exit and (expected_exit!=0 or (ran['exit']==0 and ran['stdout']==expected))
       results.append({'lane':lane,'macro':macro,'input':value,'passed':good,'build_exit':built['exit'],'expected_exit':expected_exit,'stdout':ran['stdout'] if ran else None,'expected_stdout':expected if expected_exit==0 else None,'daemon_alive_after_request':daemon.poll() is None})
     finally:
      if daemon.poll() is None:
       daemon.terminate()
       try:daemon.wait(timeout=15)
       except subprocess.TimeoutExpired:daemon.kill();daemon.wait(timeout=10)
except Exception as error:
 records.append({'exception':repr(error)});raise
finally:
 (root/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
 report={'schema':'lustr.cache-repair-daemon.v1','checks':results,'passed':len(results)==18 and all(x['passed'] and x['daemon_alive_after_request'] for x in results)}
 (root/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'checks':len(results),'passed':report['passed']},indent=2))
raise SystemExit(0 if report['passed'] else 1)
