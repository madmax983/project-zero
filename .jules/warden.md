# Warden's Journal

**2024-10-24 - [Initialization]**
**Threat:** Initial Security Audit
**Defense:** Created journal. Scanning for vulnerabilities.

**2024-10-24 - [DoS in Input Handling]**
**Threat:** Integer overflow panic in `handle_input` for `Viewport` and `BuildMode` cursor.
**Defense:** Switched to `wrapping_*` for Viewport and `saturating_*` for Cursor.
