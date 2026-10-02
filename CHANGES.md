# Changes

Every commit, newest first, grouped by day. Times are Pacific
(UTC-07:00), as committed.

Categories: `feat` new capability, `fix` a bug or wrong behavior,
`refactor` structure without behavior change, `test` tests only,
`build` build and tooling, `ext` an extension or a change to one,
`abi` the native ABI, SDK or loader, `docs` documentation, `plan`
saga planning and reordering, `release` milestone release, `chore`
agentrail bookkeeping (step complete, saga archive), `vendor` a
refresh of the vendored X_eTaL.

## 2026-10-02

- 12:09 `build` Vendoring: `just vendor [REF]` snapshots a committed ref of ../X_eTaL into vendor/xetal/ (VENDORED records it); `just xetal`, `xetal-version`, `eval`, `check-vendor` (the CLI builds, answers, names the vendored commit, runs life.xtl, imports Stats); in the gate. Scripts as in ../X_eTaL-libraries.
- 12:05 `vendor` X_eTaL 7b70575 vendored.

- 11:15 `plan` Scaffold: the agentrail process (saga foundation), CLAUDE.md/AGENTS.md, README, COPYRIGHT, LICENSE, CHANGES.md, justfile, the gate, docs/plan.md (architecture A1-A14: vendored X_eTaL, ABI V1, SDK, loader, the `ext:` bridge host until X_eTaL has a native hook, facades; the catalog of 8 extensions; four sagas), docs/xetal-asks.md (E1 native hook, E2 `.xtlm`), docs/research.txt (archival copy).

- 08:45 `chore` First commit: an empty README.
