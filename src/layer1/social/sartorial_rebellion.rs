use crate::layer1::social::factions::FactionId;
use crate::layer1::social::factions::FactionMember;
use crate::layer1::unrest::Unrest;
use bevy_ecs::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum SignifierType {
    RedBandana,
    BackwardsCap,
    RolledSleeves,
}

#[derive(Component)]
pub struct VisualSignifier {
    pub signifier: SignifierType,
}

#[derive(Component, Default)]
pub struct Appearance {
    pub signifiers: Vec<SignifierType>,
}

#[derive(Resource, Default)]
pub struct DressCodePolicy {
    pub banned_signifiers: Vec<SignifierType>,
}

#[derive(Resource, Default)]
pub struct FactionSignifiers {
    pub map: std::collections::HashMap<FactionId, SignifierType>,
}

pub fn adopt_visual_signifier_system(
    mut pops: Query<(&FactionMember, &mut Appearance)>,
    faction_signifiers: Res<FactionSignifiers>,
    policy: Option<Res<DressCodePolicy>>,
) {
    for (member, mut appearance) in pops.iter_mut() {
        if let Some(faction_id) = member.faction_id {
            if let Some(signifier) = faction_signifiers.map.get(&faction_id) {
                // Do not adopt if it's currently banned
                let is_banned = policy
                    .as_ref()
                    .is_some_and(|p| p.banned_signifiers.contains(signifier));
                if !appearance.signifiers.contains(signifier) && !is_banned {
                    appearance.signifiers.push(signifier.clone());
                }
            }
        }
    }
}

pub fn enforce_dress_code_system(
    policy: Option<Res<DressCodePolicy>>,
    mut pops: Query<&mut Appearance>,
    mut unrest: Option<ResMut<Unrest>>,
) {
    if let Some(policy) = policy {
        for mut appearance in pops.iter_mut() {
            let mut removed = false;
            appearance.signifiers.retain(|s| {
                if policy.banned_signifiers.contains(s) {
                    removed = true;
                    false
                } else {
                    true
                }
            });

            if removed {
                if let Some(ref mut unrest) = unrest {
                    unrest.level += 0.05; // Penalty for suppressing expression
                    unrest.level = unrest.level.clamp(0.0, 1.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    fn setup_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        // Run enforce dress code *before* or *after* doesn't matter too much if adopt respects policy,
        // but let's just make sure both run. Actually, we should test them somewhat sequentially to be sure.
        app.add_systems(
            bevy_app::Update,
            (
                enforce_dress_code_system,     // Remove banned ones
                adopt_visual_signifier_system, // Try to adopt (will fail if banned)
            )
                .chain(),
        );
        app
    }

    #[test]
    fn test_pop_adopts_faction_signifier() {
        let mut app = setup_app();

        let mut faction_signifiers = FactionSignifiers::default();
        faction_signifiers
            .map
            .insert(FactionId::MinersGuild, SignifierType::RedBandana);
        app.insert_resource(faction_signifiers);

        // Spawn a pop belonging to that faction, without the signifier initially
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                Appearance { signifiers: vec![] },
            ))
            .id();

        app.update();

        // The pop should have adopted the signifier
        let appearance = app.world().get::<Appearance>(pop).unwrap();
        assert!(appearance.signifiers.contains(&SignifierType::RedBandana));
    }

    #[test]
    fn test_banning_signifier_increases_unrest() {
        let mut app = setup_app();

        // Insert global dress code policy
        app.insert_resource(DressCodePolicy {
            banned_signifiers: vec![SignifierType::RedBandana],
        });

        let mut faction_signifiers = FactionSignifiers::default();
        faction_signifiers
            .map
            .insert(FactionId::MinersGuild, SignifierType::RedBandana);
        app.insert_resource(faction_signifiers);

        app.insert_resource(Unrest {
            level: 0.10,
            modifiers: vec![],
        });

        // Spawn a pop with the banned signifier ALREADY adopted (from before the ban)
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                Appearance {
                    signifiers: vec![SignifierType::RedBandana],
                },
            ))
            .id();

        app.update();

        // The signifier should be removed, but unrest should increase
        let appearance = app.world().get::<Appearance>(pop).unwrap();
        assert!(!appearance.signifiers.contains(&SignifierType::RedBandana));

        let unrest = app.world().resource::<Unrest>();
        assert!(
            unrest.level > 0.10,
            "Unrest should increase when signifier is banned"
        );
    }
}
