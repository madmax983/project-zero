echo '1. Claim work'
sed -i 's/- \[ \] `1120` Xenoflora Pet Craze.*//g' design/BACKLOG.md
echo '- [ ] `1120` Xenoflora Pet Craze — `specs/1120-xenoflora-pet-craze.md` — claimed 2026-02-01' >> design/IN_PROGRESS.md
