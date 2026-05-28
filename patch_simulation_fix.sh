cat << 'INNER_EOF' > /tmp/merge_sim_fix.diff
<<<<<<< SEARCH
    crate::experimental::whispering_well::register(schedule);
    crate::experimental::symbiotic_spores::register(schedule);
    crate::experimental::panic_buying::register(schedule);
=======
    crate::experimental::whispering_well::register(schedule);
    crate::experimental::symbiotic_spores::register(schedule);
>>>>>>> REPLACE
INNER_EOF
patch src/simulation.rs < /tmp/merge_sim_fix.diff
