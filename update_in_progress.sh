#!/bin/bash
sed -i '/- \[ \].*254/d' design/BACKLOG.md
echo '- [ ] `254` Doppelgangers — `specs/254-doppelgangers.md` — claimed '$(date +%Y-%m-%d) >> design/IN_PROGRESS.md
