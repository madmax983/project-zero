**[TUI String Allocations]**
**Learning:** Ratatui `Span::styled` accepts `Into<Cow<'a, str>>`. Using `String` for constant characters (like map tiles) allocates excessively per frame.
**Action:** Use `&'static str` for constant UI elements and pass references to `Span` to avoid heap traffic.

**[O(N) Lookups in Render Loop]**
**Learning:** Iterating `Vec` inside a screen-space render loop creates O(Pixels * Entities) complexity. Even with small N, it kills scalability.
**Action:** Convert entity lists to `HashMap` or `Grid` for O(1) spatial lookups before rendering.
