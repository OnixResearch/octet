#!/usr/bin/env python3
"""Check completed producer tasks and execute only reviewed native spec sync."""
import json
from pathlib import Path
import shutil,subprocess,time
BASE=Path('/home/brittonr/mantle-cairn-finish')
SOURCE=BASE/'xrepo/octet-mantle-producer-20261020'
RUN=BASE/'runs/octet-native-sync-resume31'; RUN.mkdir(exist_ok=False)
CLI=Path('/nix/store/ynjawpza9ci9i8ha7f3ywaqwbd60m83k-cairn-0.1.0/bin/cairn')
CHANGE='publish-mantle-producer-handoff'; ACTIVE=SOURCE/'.cairn/changes'/CHANGE
records=[]
def capture(label,args):
    args=list(map(str,args)); start=time.monotonic()
    with (RUN/(label+'.stdout')).open('xb') as out,(RUN/(label+'.stderr')).open('xb') as err:
        result=subprocess.run(args,cwd=SOURCE,stdout=out,stderr=err,timeout=600)
    row=dict(label=label,command=args,cwd=str(SOURCE),exit=result.returncode,elapsed_s=time.monotonic()-start)
    records.append(row); (RUN/'results.json').write_text(json.dumps(records,indent=2)+chr(10));print(json.dumps(row),flush=True)
    if result.returncode: raise SystemExit(result.returncode)
    return json.loads((RUN/(label+'.stdout')).read_text())
assert capture('native-validate',[CLI,'validate','--root','.'])['valid']
for gate in ['proposal','design','tasks']:
    value=capture('native-gate-'+gate,[CLI,'gate',gate,CHANGE,'--root','.'])
    assert value['verdict']=='PASS'
plan=capture('native-sync-plan',[CLI,'sync',CHANGE,'--root','.'])
assert not plan['blocked'] and plan['reasons']==[]
documents=plan['merge_preflight']['documents']
assert len(documents)==1 and documents[0]['destination']=='.cairn/specs/mantle-producer-handoff/spec.md'
assert {operation['requirement_id'] for operation in documents[0]['operations']}=={'octet.mantle_producer_handoff.contract','octet.mantle_producer_handoff.acquisition','octet.mantle_producer_handoff.boundary'}
assert {operation['kind'] for operation in documents[0]['operations']}=={'added'}
assert capture('native-sync-execute',[CLI,'sync',CHANGE,'--root','.','--execute'])['mutated']
post=capture('native-post-sync-plan',[CLI,'sync',CHANGE,'--root','.'])
assert not post['blocked'] and all(not document['changed'] for document in post['merge_preflight']['documents'])
archive=capture('native-archive-plan',[CLI,'archive',CHANGE,'--root','.'])
assert not archive['blocked'] and archive['reasons']==[]
assert [action['kind'] for action in archive['actions']]==['archive_change']
(RUN/'complete.json').write_text(json.dumps(dict(records=records,sync_executed=True,sync_idempotent=True,archive_preview_blocked=False,archive_executed=False,release_authority=False),indent=2)+chr(10))
evidence=ACTIVE/'evidence/native-terminal-v31';evidence.mkdir()
for prior in ['octet-native-readiness-resume30','octet-native-sync-resume31']:
    target=evidence/prior; target.mkdir()
    for path in (BASE/'runs'/prior).iterdir():
        if path.is_file(): shutil.copy2(path,target/path.name)
shutil.copy2(BASE/'bin/jobs/octet-native-readiness-resume30.py',evidence/'readiness-driver.py')
shutil.copy2(BASE/'bin/jobs/octet-native-sync-resume31.py',evidence/'sync-driver.py')
