sed -i '50i \
    world.init_resource::<bevy_ecs::event::Events<crate::layer1::cassandra_syndrome::DoomsdayWarningEvent>>();\
    world.init_resource::<bevy_ecs::event::Events<crate::layer1::cassandra_syndrome::DisasterOccurredEvent>>();' src/setup.rs
