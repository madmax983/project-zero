#!/bin/bash
sed -i '/- \[ \].*254/d' design/IN_PROGRESS.md
echo '- [x] `254` Doppelgangers — `specs/254-doppelgangers.md` — completed '$(date +%Y-%m-%d) >> design/COMPLETED.md
