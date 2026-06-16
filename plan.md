1. **Append Fragments to `lore/FRAGMENTS.md`**
   - Add new fragments for Sartorial Rebellion (Spec 1284).
   - Add new fragments for Dead Protocols (Spec 1038).
   - Add new fragments for The Orphaned Swarm (Spec 1041).
   - Add new fragments for Debt of the Dead (Spec 877).
   - Add new fragments for Light & Darkness (Spec 340).
   - Add new fragments for Cassandra Syndrome (Spec 1305).
2. **Append Templates to `lore/TEMPLATES.md`**
   - Add templates for Sartorial Rebellion (Spec 1284).
   - Add templates for Dead Protocols (Spec 1038).
   - Add templates for The Orphaned Swarm (Spec 1041).
   - Add templates for Debt of the Dead (Spec 877).
   - Add templates for Light & Darkness (Spec 340).
   - Add templates for Cassandra Syndrome (Spec 1305).
   - Add templates for Architectural Superstition (Spec 1306).
3. **Append Grammars to `lore/GRAMMARS.md`**
   - Add chaining rules mapping events from the above mechanics to their consequences.
4. **Append Lexicon entries to `lore/LEXICON.md`**
   - Define vocabulary for the mechanics (e.g., Visual Signifiers, Diplomatic Beacon, Inherited Burden, Doomsday Warning).
5. **Run test suite**
   - Run `cargo test --lib` to ensure no unexpected regressions are introduced.
6. **Pre-commit step**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
7. **Submit changes**
   - Use `default_api:submit` with message "lore: add lore hooks for recently completed mechanics".
