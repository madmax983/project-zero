**[TUI String Allocations]**
**Learning:** Ratatui `Span::styled` accepts `Into<Cow<'a, str>>`. Using `String` for constant characters (like map tiles) allocates excessively per frame.
**Action:** Use `&'static str` for constant UI elements and pass references to `Span` to avoid heap traffic.
