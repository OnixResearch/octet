#!/usr/bin/env python3
"""Retain actual immutable producer evidence and inspect native terminal plans."""
import json
from pathlib import Path
import shutil
import subprocess
import time
BASE=Path('/home/brittonr/mantle-cairn-finish')
SOURCE=BASE/'xrepo/octet-mantle-producer-20261020'
RUN=BASE/'runs/octet-native-readiness-resume30'
RUN.mkdir(exist_ok=False)
CLI=Path('/nix/store/ynjawpza9ci9i8ha7f3ywaqwbd60m83k-cairn-0.1.0/bin/cairn')
CHANGE='publish-mantle-producer-handoff'
PROOF=BASE/'runs/octet-mantle-producer-acceptance-20261020-v10'
acceptance=json.loads((PROOF/'complete.json').read_text())
assert acceptance['revision']=='9237f931e8e65dd4254011839d5fd207f2ad85a5'
assert acceptance['owner_job_executed'] and acceptance['external_proof_accepted']
assert all(r['exit']==r['expected'] for r in acceptance['records'])
EVIDENCE=SOURCE/'.cairn/changes'/CHANGE/'evidence/owner-acceptance-v10'
EVIDENCE.mkdir(parents=True,exist_ok=False)
for name in ['complete.json','results.json']:
    shutil.copy2(PROOF/name,EVIDENCE/name)
for row in acceptance['records']:
    for suffix in ['stdout','stderr']:
        path=PROOF/(row['label']+'.'+suffix)
        assert path.stat().st_size<=16*1024*1024
        shutil.copy2(path,EVIDENCE/path.name)
shutil.copy2(BASE/'bin/jobs/finish-octet-producer-acceptance-v10.py',EVIDENCE/'executed-driver.py')
output=Path(acceptance['actual_owner_output'])
for name in ['producer-prove.json','producer-artifact-verification.json','external-receipt.json','external-profile.json','job.json','valence-provenance.json','source-revision']:
    shutil.copy2(output/name,EVIDENCE/name)
records=[]
def capture(label,args):
    args=list(map(str,args)); started=time.monotonic()
    with (RUN/(label+'.stdout')).open('xb') as out,(RUN/(label+'.stderr')).open('xb') as err:
        result=subprocess.run(args,cwd=SOURCE,stdout=out,stderr=err,timeout=600)
    row=dict(label=label,command=args,cwd=str(SOURCE),exit=result.returncode,elapsed_s=time.monotonic()-started)
    records.append(row); (RUN/'results.json').write_text(json.dumps(records,indent=2)+'\n'); print(json.dumps(row),flush=True)
    if result.returncode: raise SystemExit(result.returncode)
    return (RUN/(label+'.stdout')).read_text()
capture('retained-proof-output-root',['nix-store','--add-root',BASE/'retained-octet-producer-9237f931','--indirect','--realise',output])
results={}
for label,args in [('native-validation',['validate','--root','.']),('proposal-gate',['gate','proposal',CHANGE,'--root','.']),('design-gate',['gate','design',CHANGE,'--root','.']),('tasks-gate',['gate','tasks',CHANGE,'--root','.']),('sync-plan',['sync',CHANGE,'--root','.']),('archive-plan',['archive',CHANGE,'--root','.'])]:
    results[label]=json.loads(capture(label,[CLI,*args]))
    assert results[label].get('valid',True) is True
for label in ['proposal-gate','design-gate','tasks-gate']:
    assert results[label]['verdict']=='PASS'
(RUN/'complete.json').write_text(json.dumps(dict(revision=acceptance['revision'],records=records,gate_verdicts={label:results[label]['verdict'] for label in ['proposal-gate','design-gate','tasks-gate']},sync_plan_blocked=results['sync-plan'].get('blocked'),archive_plan_blocked=results['archive-plan'].get('blocked'),archive_plan_reasons=results['archive-plan'].get('reasons'),archive_executed=False,publication_implementation='9237f931e8e65dd4254011839d5fd207f2ad85a5',release_authority=False),indent=2)+'\n')
