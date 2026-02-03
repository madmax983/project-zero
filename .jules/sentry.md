# Sentry's Journal

## [InputRouter Mouse Scope]
**Learning:** `InputRouter` had comprehensive keyboard tests but completely lacked mouse interaction tests, leaving the `route_mouse` logic unverified against context switching.
**Action:** Always verify that routing logic (dispatchers) covers ALL input types (Key, Mouse, etc.) for ALL states (Normal, Build, Overlay).
