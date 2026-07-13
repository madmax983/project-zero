sed -i '/world.init_resource::<Events<crate::layer1::cassandra_syndrome::DoomsdayWarningEvent>>();/a \
    world.init_resource::<Events<crate::layer1::cassandra_syndrome::DisasterOccurredEvent>>();' src/simulation.rs
