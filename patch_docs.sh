#!/bin/bash
sed -i 's/- \[ \] `306` The Debt Collector — `specs\/306-debt-collector.md`//g' design/BACKLOG.md
echo "- [x] \`306\` The Debt Collector — \`specs/306-debt-collector.md\` — completed 2026-02-24" >> design/COMPLETED.md
