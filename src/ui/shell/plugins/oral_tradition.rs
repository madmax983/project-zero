use super::{render_with_frame, SharedWorld};
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui_hypertile_extras::HypertilePlugin;

pub struct OralTraditionPlugin {
    world: SharedWorld,
}

impl OralTraditionPlugin {
    pub const fn new(world: SharedWorld) -> Self {
        Self { world }
    }
}

impl HypertilePlugin for OralTraditionPlugin {
    fn render(&self, area: Rect, buf: &mut Buffer, _is_focused: bool) {
        let world = self.world.borrow();
        render_with_frame(area, buf, |frame| {
            crate::ui::oral_tradition::render_oral_tradition(frame, frame.area(), &world);
        });
    }
}
