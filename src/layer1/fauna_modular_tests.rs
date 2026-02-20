#[cfg(test)]
mod tests {
    use crate::layer1::fauna::{BodyPartType, FaunaBody, FaunaPart, FaunaStats};
    use crate::layer1::fauna_gen::{FaunaGenerator, FaunaSeed};

    #[test]
    fn test_fauna_body_generation_from_seed() {
        let seed = FaunaSeed::new(12345);
        let body = FaunaGenerator::generate(seed);

        assert!(body.parts.contains_key(&BodyPartType::Head));
        assert!(body.parts.contains_key(&BodyPartType::Body));
        assert!(body.parts.contains_key(&BodyPartType::Limbs));

        // Deterministic check
        let seed2 = FaunaSeed::new(12345);
        let body2 = FaunaGenerator::generate(seed2);
        assert_eq!(body.name, body2.name);
    }

    #[test]
    fn test_fauna_stats_aggregation() {
        // Create a body with specific parts
        let mut body = FaunaBody::default();

        // Head: Sharp Teeth (+5 Damage)
        body.add_part(FaunaPart {
            part_type: BodyPartType::Head,
            name: "Wolf Head".to_string(),
            stats: FaunaStats {
                attack: 5.0,
                ..Default::default()
            },
            resource_drop: None,
        });

        // Body: Thick Hide (+10 Health)
        body.add_part(FaunaPart {
            part_type: BodyPartType::Body,
            name: "Bear Torso".to_string(),
            stats: FaunaStats {
                health_max: 10.0,
                ..Default::default()
            },
            resource_drop: None,
        });

        let stats = body.aggregate_stats();

        assert_eq!(stats.attack, 5.0);
        assert_eq!(stats.health_max, 10.0);
    }

    #[test]
    fn test_legacy_fauna_type_mapping() {
        // Ensure old enum still works or converts
        let wolf_seed = FaunaSeed::from_preset("Wolf");
        let body = FaunaGenerator::generate(wolf_seed);

        let stats = body.aggregate_stats();
        assert!(stats.attack > 0.0);
        assert!(body.name.contains("Wolf") || body.name.contains("Grey Wolf"));
    }

    #[test]
    fn test_husbandry_resource_check() {
        // Cow-like creature should produce Milk
        let mut body = FaunaBody::default();
        body.add_part(FaunaPart {
            part_type: BodyPartType::Body,
            name: "Udder-Body".to_string(),
            resource_drop: Some("Milk".to_string()),
            ..Default::default()
        });

        assert!(body.can_produce("Milk"));
        assert!(!body.can_produce("Wool"));
    }
}
