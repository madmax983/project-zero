#!/bin/bash
# Move 901 from BACKLOG to COMPLETED
sed -i '/- \[ \] `901`/d' design/BACKLOG.md
echo "- [x] \`901\` Derelict Stations — \`specs/901-derelict-stations.md\` — completed $(date +%Y-%m-%d)" >> design/COMPLETED.md
