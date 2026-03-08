sed -i 's/- \[ \] `350` Private Stashes — `specs\/350-private-stashes.md`//' design/BACKLOG.md
echo "- [ ] \`350\` Private Stashes — \`specs/350-private-stashes.md\` — claimed $(date +%Y-%m-%d)" >> design/IN_PROGRESS.md
git add design/
git commit -m "claim: 350 private stashes"
