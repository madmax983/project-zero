cat << 'INNER_EOF' > /tmp/merge_int.diff
<<<<<<< SEARCH
mod temporal_echoes_maintenance;
=======
mod temporal_echoes_maintenance;
mod celestial_library_bridge;
>>>>>>> REPLACE
INNER_EOF
patch tests/integration.rs < /tmp/merge_int.diff
