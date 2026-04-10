**2024-05-28 - [Path Traversal in Cartography Export]
**Threat:** [Path Traversal] The `CartographyExportConfig` allowed an unsanitized `String` to dictate the file output path, enabling writes to arbitrary files (e.g. `../../../etc/passwd`).
**Defense:** [Parse, don't validate] Replaced the raw `String` with an `ExportPath` newtype wrapper. Its constructor strictly validates that the path string contains no path separators (`/`, `\`) or traversal elements (`..`), and only allows alphanumeric characters, dots, dashes, and underscores.

**2026-04-10 - [DoS Mitigation in UI Shell Rendering]
**Threat:** [Denial of Service] The UI rendering system for shell plugins used `.expect()` when instantiating and drawing to offscreen `Terminal`s. If backend initialization or drawing failed (e.g., terminal I/O errors, resource limits), the entire application would panic.
**Defense:** [Graceful Error Handling] Replaced `.expect()` calls with safe `match` and `if let Err` blocks that log errors using `log::error!` and return early, preventing application crashes.
