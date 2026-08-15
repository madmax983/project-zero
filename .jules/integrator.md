## Integration Insights
**Task:** Pulsar Timing (INT-1143)
**Key Learnings:** Remember to use `.chain()` explicitly when registering sequential integration systems into Bevy's schedule to prevent subtle 1-tick frame execution bugs, aligning with the Integrator Persona rules and satisfying Code Review constraints.
