sed -i '/^\}$/i \
    schedule.add_systems(\
        (\
            crate::layer1::cassandra_syndrome::generate_doomsday_warning,\
            crate::layer1::cassandra_syndrome::handle_ignored_warning,\
            crate::layer1::cassandra_syndrome::validate_prophecy,\
        )\
            .chain()\
            .in_set(Layer1SystemSet::Execution),\
    );\
' src/layer1/systems/execution.rs
