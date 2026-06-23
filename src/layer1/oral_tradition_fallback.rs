use crate::layer1::core::chronicle::Chronicle;
use bevy_ecs::prelude::Resource;

#[derive(Debug)]
pub struct Story {
    pub text: String,
    pub historical_date: u64,
    pub mutations: u32,
    pub genre: StoryGenre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryGenre {
    Heroic,
    Tragedy,
    Cautionary,
    Trivial,
}

#[derive(Debug, Default, Resource)]
pub struct OralTradition {
    pub stories: Vec<Story>,
}

use std::sync::atomic::{AtomicBool, Ordering};

static WARNING_PRINTED: AtomicBool = AtomicBool::new(false);

fn print_nova_warning() {
    if !WARNING_PRINTED.swap(true, Ordering::SeqCst) {
        use comfy_table::{presets::UTF8_FULL, Attribute, Cell, Color as TableColor, Table};
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new(" ✗ FEATURE DISABLED ")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Red),
                Cell::new("Details").add_attribute(Attribute::Bold),
            ])
            .add_row(vec![
                Cell::new("Message")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Yellow),
                Cell::new("⚠️ Oral Tradition is disabled. You are seeing dummy data.")
                    .fg(TableColor::Red),
            ])
            .add_row(vec![
                Cell::new("Action")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Yellow),
                Cell::new("Enable feature 'nova' in Cargo.toml to use Oral Tradition.")
                    .fg(TableColor::Cyan),
            ]);
        eprintln!("{table}");
    }
}

impl OralTradition {
    pub fn add_story(&mut self, story: Story) {
        self.stories.push(story);
        print_nova_warning();
    }

    pub fn process_chronicles(&mut self, _chronicle: &Chronicle) {
        print_nova_warning();
    }
}
