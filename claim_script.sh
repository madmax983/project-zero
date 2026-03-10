sed -i '/256.*placebo-protocols/d' design/BACKLOG.md
echo '- [ ] `256` Placebo Protocols — `specs/256-placebo-protocols.md` — claimed 2026-03-06' >> design/IN_PROGRESS.md
git add design/
git commit -m "claim: 256 placebo protocols"
