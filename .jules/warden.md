**2024-05-28 - [Path Traversal in Cartography Export]
**Threat:** [Path Traversal] The `CartographyExportConfig` allowed an unsanitized `String` to dictate the file output path, enabling writes to arbitrary files (e.g. `../../../etc/passwd`).
**Defense:** [Parse, don't validate] Replaced the raw `String` with an `ExportPath` newtype wrapper. Its constructor strictly validates that the path string contains no path separators (`/`, `\`) or traversal elements (`..`), and only allows alphanumeric characters, dots, dashes, and underscores.
