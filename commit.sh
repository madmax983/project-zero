git add .
git commit -m "$(cat <<'INNER_EOF'
feat(layer1): complete architecture of paranoia system

Implements RED-GREEN-REFACTOR from spec 1326:
- Added comprehensive test suite (RED phase)
- Implemented Subversive, Surveillance components and spread/morale systems (GREEN phase)
- Test coverage: >= 85%

All acceptance criteria met:
- Subversion spreads to adjacent pops
- Surveillance blocks subversion spread
- Surveillance lowers pop morale
- cargo test passes
- cargo clippy clean

Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>
INNER_EOF
)"
