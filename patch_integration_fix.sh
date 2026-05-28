cat << 'INNER_EOF' > /tmp/merge_fix.diff
<<<<<<< SEARCH
/// Updates `AstrologicalBelief` based on the Layer 2 `SyzygyCycle`.
// --- INT-310: Celestial Library -> ColonyResources & Chronicle ---
=======
// --- INT-310: Celestial Library -> ColonyResources & Chronicle ---
>>>>>>> REPLACE
INNER_EOF
patch src/layer2/integration.rs < /tmp/merge_fix.diff
