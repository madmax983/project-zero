cat << 'INNER_EOF' > /tmp/merge.diff
<<<<<<< SEARCH
        crate::layer2::integration::logistics_strained_chronicle_bridge
=======
        crate::layer2::integration::celestial_library_chronicle_bridge,
        crate::layer2::integration::logistics_strained_chronicle_bridge
>>>>>>> REPLACE
<<<<<<< SEARCH
        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();
=======
        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();
        world.init_resource::<Events<crate::layer2::celestial_library::LibraryDonationEvent>>();
>>>>>>> REPLACE
INNER_EOF
patch src/simulation.rs < /tmp/merge.diff
