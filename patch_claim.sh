sed -i 's/- \[ \] `623` The Nostalgia Plague — `specs\/623-nostalgia-plague.md`//g' design/BACKLOG.md
sed -i '/---/a - [ ] `623` The Nostalgia Plague — `specs\/623-nostalgia-plague.md` — claimed 2026-02-01' design/IN_PROGRESS.md
git add design/BACKLOG.md design/IN_PROGRESS.md
git commit -m "claim: 623 nostalgia plague"
