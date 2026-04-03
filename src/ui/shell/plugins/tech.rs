use super::{render_with_frame, SharedWorld};
use crate::ui::tech::TechUiState;
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui_hypertile_extras::HypertilePlugin;

pub struct TechPlugin {
    world: SharedWorld,
}

impl TechPlugin {
    pub const fn new(world: SharedWorld) -> Self {
        Self { world }
    }

    fn set_open_state(&self, is_open: bool) -> bool {
        let mut world = self.world.borrow_mut();
        let mut ui_state = world.resource_mut::<TechUiState>();
        let previous = ui_state.is_open;
        ui_state.is_open = is_open;
        previous
    }
}

impl HypertilePlugin for TechPlugin {
    fn render(&self, area: Rect, buf: &mut Buffer, _is_focused: bool) {
        let previous = self.set_open_state(true);
        {
            let world = self.world.borrow();
            render_with_frame(area, buf, |frame| {
                crate::ui::tech::render_tech_tree(frame, frame.area(), &world);
            });
        }
        let _ = self.set_open_state(previous);
    }
}
