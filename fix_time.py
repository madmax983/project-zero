import re

with open('src/layer1/gravitational_debt.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''pub fn gravitational_debt_accumulation_system(
    time: Res<Time>,
    mut query: Query<(&AntiGravGenerator, &PowerConsumer, &mut GravitationalDebt)>,
) {
    let delta = time.delta_secs();''',
'''pub fn gravitational_debt_accumulation_system(
    time: Res<Time>,
    mut query: Query<(&AntiGravGenerator, &PowerConsumer, &mut GravitationalDebt)>,
) {
    let delta = time.delta_secs();'''
)

content = content.replace(
'''    #[test]
    fn test_gravitational_debt_accumulates_when_powered() {
        let mut app = App::new();
        app.add_plugins(bevy_time::TimePlugin);
        app.update();
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));

        app.add_systems(Update, gravitational_debt_accumulation_system);''',
'''    #[test]
    fn test_gravitational_debt_accumulates_when_powered() {
        let mut app = App::new();
        let mut time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        app.add_systems(Update, gravitational_debt_accumulation_system);'''
)

with open('src/layer1/gravitational_debt.rs', 'w') as f:
    f.write(content)
