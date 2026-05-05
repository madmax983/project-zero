sed -i 's/- \[ \] `631` Conveyor Logistics — `specs\/631-conveyor-logistics.md`//g' design/BACKLOG.md
echo "- [ ] \`631\` Conveyor Logistics — \`specs/631-conveyor-logistics.md\` — claimed $(date +%Y-%m-%d)" >> design/IN_PROGRESS.md
git add design/
git commit -m "claim: 631 conveyor logistics"
