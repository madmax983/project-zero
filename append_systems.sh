cat << 'INNER_EOF' >> src/layer1/systems/execution.rs

pub fn xenoflora_systems(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (
            crate::layer1::social::xenoflora_pet::pet_resource_consumption_system,
            crate::layer1::social::xenoflora_pet::pet_viral_spread_system,
            crate::layer1::social::xenoflora_pet::apply_pet_mood_boost,
        )
            .chain()
            .in_set(Layer1Systems::Social)
            .run_if(bevy_ecs::schedule::in_state(crate::GameState::Playing))
            .run_if(bevy_time::common_conditions::on_timer(std::time::Duration::from_secs_f32(0.5))),
    );
}
INNER_EOF
