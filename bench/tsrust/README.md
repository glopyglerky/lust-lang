# bench/tsrust

The rustc numbers lust must beat on ts-rust (786k lines), and the harness
that produced them.

- `baseline.json`: the checked-in scoreboard, copied from the measured
  baseline report (`/tmp/lust-ts/baseline/BASELINE.md`, 2026-10-12,
  ts-rust commit `b11b618a`). Each row has the best rustc config, its time,
  and the 100x target. `lust_s` is null: lust cannot compile ts-rust yet.
- `edits.py`: the paired edit list (body edit, new fn, signature change,
  type change, mistake, fix). The same edits will drive the lust column.
- `timeit.py`: the timing harness from the baseline run.
- `run.sh`: opt-in rerun on a copy of ts-rust. Never edits the original.
  A full pass takes tens of minutes, so nothing runs it by default,
  and CI does not run it at all.

Rerun:

```sh
TSRUST_RERUN=1 bench/tsrust/run.sh            # cold rows + edit loop
TSRUST_RERUN=1 TSRUST_ROWS=editloop bench/tsrust/run.sh
```

To add the lust column later: in `run.sh`, replace the stub block at the
end with paired lust runs over the same copy and the same `edits.py`
sequence, then fill `lust_s` in `baseline.json` from a real rerun.
Do not paste numbers from other corpora into these rows.
