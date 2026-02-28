//! SCALE WASM entry point — runs the game in a browser via Ratzilla.

#[cfg(target_arch = "wasm32")]
use ratzilla::backend::dom::DomBackend;
#[cfg(target_arch = "wasm32")]
use ratzilla::ratatui::Terminal;
#[cfg(target_arch = "wasm32")]
use ratzilla::WebRenderer;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This binary is for WASM only.");
}

#[cfg(target_arch = "wasm32")]
fn main() -> std::io::Result<()> {
    let world = Rc::new(RefCell::new(setup_world()));

    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    // Key input handler
    terminal.on_key_event({
        let world = world.clone();
        move |key_event| {
            if let Ok(game_key) = GameKeyEvent::try_from(key_event) {
                route_input(&mut world.borrow_mut(), game_key);
            }
        }
    });

    // Mouse input handler
    terminal.on_mouse_event({
        let world = world.clone();
        move |mouse_event| {
            if let Ok(game_mouse) = GameMouseEvent::try_from(mouse_event) {
                route_mouse_input(&mut world.borrow_mut(), game_mouse);
            }
        }
    });

    // Render loop (runs at ~60fps via requestAnimationFrame)
    let frame_count = Rc::new(RefCell::new(0u32));

    terminal.draw_web({
        let world = world;
        move |frame| {
            let mut world = world.borrow_mut();

            // Update wall time (approx 60fps)
            world.resource_mut::<WallTime>().0 += 1.0 / 60.0;

            // Rate-limit simulation: every 6 frames ≈ 10 ticks/sec at 60fps
            let mut count = frame_count.borrow_mut();
            *count += 1;

            if *count >= 6 {
                *count = 0;
                if *world.resource::<GameState>() == GameState::Running {
                    let speed = world.resource::<SimulationTime>().speed;
                    if speed != SimSpeed::Paused {
                        run_simulation_tick(&mut world);
                    }
                }
            }

            update_camera_smooth(&mut world);
            update_render_cache(&mut world);
            render(&world, frame);
        }
    });

    Ok(())
}
