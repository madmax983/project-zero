#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use scale::shared::narrative::{NarrativeContext, NarrativeGenerator};

    #[test]
    fn test_lore_parsing() {
        let generator = NarrativeGenerator::from_embedded();
        let context = NarrativeContext::new();

        // Check for new templates by attempting to generate from them.
        // We don't care about the output string, just that it doesn't error (meaning template exists).

        // These might fail if the template requires slots that are not in context AND don't have fragment fallbacks.
        // Looking at my templates:
        // MERCHANT_ARRIVAL: [COLONY], [YEAR], [MERCHANT_TITLE]
        // [MERCHANT_TITLE] is a fragment, so it should resolve.
        // [COLONY], [YEAR] are expected context. If missing, the generator currently outputs "[COLONY]" literal?
        // Let's check the code:
        // if let Some(val) = context.get(key) ... else if let Some(fragment) ... else { output.push('['); ... }
        // So it won't error if context is missing, it just prints the placeholder.
        // The only error `generate` returns is "Template not found" or "Template has no patterns".

        assert!(
            generator.generate("MERCHANT_ARRIVAL", &context).is_ok(),
            "MERCHANT_ARRIVAL template missing"
        );
        assert!(
            generator.generate("TRADE_COMPLETED", &context).is_ok(),
            "TRADE_COMPLETED template missing"
        );
        assert!(
            generator.generate("NOISE_COMPLAINT", &context).is_ok(),
            "NOISE_COMPLAINT template missing"
        );
        assert!(
            generator.generate("QUIET_MOMENT", &context).is_ok(),
            "QUIET_MOMENT template missing"
        );
        assert!(
            generator.generate("FUNERAL_HELD", &context).is_ok(),
            "FUNERAL_HELD template missing"
        );
        assert!(
            generator.generate("EDICT_ISSUED", &context).is_ok(),
            "EDICT_ISSUED template missing"
        );
        assert!(
            generator.generate("EDICT_REVOKED", &context).is_ok(),
            "EDICT_REVOKED template missing"
        );

        // New templates added
        assert!(
            generator.generate("VERMIN_OUTBREAK", &context).is_ok(),
            "VERMIN_OUTBREAK template missing"
        );
        assert!(
            generator.generate("MILITIA_MUSTER", &context).is_ok(),
            "MILITIA_MUSTER template missing"
        );
        assert!(
            generator.generate("FAUNA_SIGHTING", &context).is_ok(),
            "FAUNA_SIGHTING template missing"
        );
        assert!(
            generator.generate("STRUCTURE_COLLAPSE", &context).is_ok(),
            "STRUCTURE_COLLAPSE template missing"
        );
        assert!(
            generator.generate("RUIN_DISCOVERY", &context).is_ok(),
            "RUIN_DISCOVERY template missing"
        );

        // New templates added for Energy, Visitors, Factions, Atmosphere
        assert!(
            generator.generate("POWER_OUTAGE", &context).is_ok(),
            "POWER_OUTAGE template missing"
        );
        assert!(
            generator.generate("VISITOR_ARRIVAL", &context).is_ok(),
            "VISITOR_ARRIVAL template missing"
        );
        assert!(
            generator.generate("FACTION_FORMED", &context).is_ok(),
            "FACTION_FORMED template missing"
        );
        assert!(
            generator.generate("ATMOSPHERE_EVENT", &context).is_ok(),
            "ATMOSPHERE_EVENT template missing"
        );

        println!("Lore parsing successful!");
    }
}
