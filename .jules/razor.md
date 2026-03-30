## [Reduction]
**Bloat:** Orphaned doc comments in `src/layer1/mod.rs` (e.g., `Justice system (Spec 072).`, `Penal labor system (Spec 118).`) that were separated from any code, triggering `clippy::empty_line_after_doc_comments` and falsely attaching themselves to unrelated modules if simply joined.
**Cut:** Deleted the dangling/orphaned doc comments entirely.
**Saved:** 6 lines of useless code / Cognitive load of misleading documentation.