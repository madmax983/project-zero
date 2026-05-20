sed -i 's/- \[ \] `1062` The Biological Stock Market — `specs\/1062-the-biological-stock-market.md`//' design/BACKLOG.md
echo "- [ ] \`1062\` The Biological Stock Market — \`specs/1062-the-biological-stock-market.md\` — claimed 2026-05-20" >> design/IN_PROGRESS.md
git add design/
git commit -m "claim: 1062 the biological stock market"
