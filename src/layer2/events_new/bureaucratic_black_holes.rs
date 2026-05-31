use bevy::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
pub struct BureaucraticComplexity(pub f32); // 0.0 to 1.0

#[derive(Component)]
pub struct Shipment {
    pub target: Entity,
    pub items: u32,
    pub is_misfiled: bool,
}

#[derive(Event)]
pub struct AuditEvent;

pub fn process_shipments_system(
    complexity: Res<BureaucraticComplexity>,
    mut shipments: Query<&mut Shipment>,
) {
    let mut rng = rand::thread_rng();
    for mut shipment in shipments.iter_mut() {
        if !shipment.is_misfiled && rng.gen_bool(complexity.0.into()) {
            shipment.is_misfiled = true;
        }
    }
}

pub fn audit_event_system(
    mut audit_events: EventReader<AuditEvent>,
    mut misfiled_items: Query<&mut Shipment>,
) {
    let mut has_audit = false;
    for _ in audit_events.read() {
        has_audit = true;
    }
    if has_audit {
        for mut shipment in misfiled_items.iter_mut() {
            if shipment.is_misfiled {
                shipment.is_misfiled = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_complexity_causes_misfiling() {
        let mut app = App::new();
        app.add_systems(Update, process_shipments_system);

        app.world_mut().insert_resource(BureaucraticComplexity(1.0)); // 100% complexity for deterministic test

        let target = app.world_mut().spawn_empty().id();
        let shipment_id = app.world_mut().spawn(Shipment { target, items: 100, is_misfiled: false }).id();

        app.update();

        let shipment = app.world().get::<Shipment>(shipment_id).unwrap();
        assert!(shipment.is_misfiled, "Shipments should be misfiled in highly complex bureaucracies");
    }

    #[test]
    fn test_audit_event_restores_misfiled_entities() {
        let mut app = App::new();
        app.add_systems(Update, audit_event_system);

        let target = app.world_mut().spawn_empty().id();
        let misfiled_shipment = app.world_mut().spawn(Shipment { target, items: 100, is_misfiled: true }).id();

        app.world_mut().insert_resource(Events::<AuditEvent>::default());
        let mut events = app.world_mut().get_resource_mut::<Events<AuditEvent>>().unwrap();
        events.send(AuditEvent);

        app.update();

        let shipment = app.world().get::<Shipment>(misfiled_shipment).unwrap();
        assert!(!shipment.is_misfiled, "Audit should clear the misfiled status");
    }
}
