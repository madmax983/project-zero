sed -i 's/- \[ \] `350` Private Stashes — `specs\/350-private-stashes.md` — claimed [0-9-]*//' design/IN_PROGRESS.md
echo "- [x] \`350\` Private Stashes — \`specs/350-private-stashes.md\` — completed $(date +%Y-%m-%d)" >> design/COMPLETED.md
