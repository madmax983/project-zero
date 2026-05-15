#!/bin/bash
sed -i 's/- \[ \] `480` Memory Forgery — `specs\/480-memory-forgery.md`//g' design/BACKLOG.md
echo "- [x] \`480\` Memory Forgery - \`specs/480-memory-forgery.md\` - completed 2026-04-10" >> design/COMPLETED.md
