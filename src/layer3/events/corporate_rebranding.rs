use bevy_ecs::prelude::*;
use crate::layer1::architecture::structure::Structure;
use crate::layer1::building::BuildingType;
// Note: Faction is not exported from layer3::faction, but there is layer3::diplomacy::Faction, we'll see where Faction is
// And the spec uses `crate::layer3::faction::Faction` and `StructureType::Headquarters` which we need to adapt to existing codebase.
