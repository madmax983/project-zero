#!/bin/bash
sed -i 's/- \[ \] `1101` Bureaucratic Ghost Towns.*/- [ ] `1101` Bureaucratic Ghost Towns — `specs\/1101-bureaucratic-ghost-towns.md`/' design/BACKLOG.md
sed -i 's/- \[ \] `1101` Bureaucratic Ghost Towns.*/- [ ] `1101` Bureaucratic Ghost Towns — `specs\/1101-bureaucratic-ghost-towns.md` — claimed '"$(date +%Y-%m-%d)"'/' design/IN_PROGRESS.md
