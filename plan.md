1. Claim task `1085` in `design/IN_PROGRESS.md` and remove from `design/BACKLOG.md`.
2. Following the **TDD Rule**, I will add the `HyperValuable` variant to `ResourceType` in `src/layer1/economy/resources.rs`.
3. Add `ResourceMinedEvent` in `src/layer1/economy/resources.rs` since it's already generated or might be added there. But wait, `ResourceMinedEvent` might conflict with `MiningEvent` if we are not careful. The spec says to use `ResourceMinedEvent`. However, since `ResourceMinedEvent` doesn't exist, we will add it to `src/layer1/economy/resources.rs`. Wait, I will add `ResourceMinedEvent` and `ResourceType::HyperValuable` to `src/layer1/economy/resources.rs`. Let's check `MiningEvent` in `src/layer1/economy/resources.rs`, it just tracks progress, not resource type.
4. I will create `PirateThreatLevel` in `src/layer3/pirates.rs` as a `Resource`.
5. I will create `process_hyper_resources` in `src/layer3/pirates.rs` and the RED phase test `test_hyper_valuable_resource_discovery_triggers_attention` in `src/layer3/pirates.rs`.
    * I'll adapt the RED Phase test to work with the existing `DiplomaticRelations` component which contains `Vec<DiplomaticStanding>`. Actually, since the RED Phase test explicitly creates `app.init_resource::<DiplomaticRelations>();` and modifies `standing: f32`, it conflicts with the `DiplomaticRelations` component in `src/layer3/diplomacy_reflection.rs`.
    * Wait, let's look at the RED Phase again. The Builder prompt states: "Write the SIMPLEST code to make tests pass... implement the spec, nothing more." So I should just create the system in a new file `src/layer3/resource_curse.rs` or directly in `src/layer3/pirates.rs` as the spec explicitly defines:
      ```rust
      pub fn process_hyper_resources(
          mut events: EventReader<ResourceMinedEvent>,
          mut diplomacy: Query<&mut DiplomaticRelations>, // Or just use ResMut if I want to strictly follow spec, but it conflicts with the actual code base. I should adapt it to the real `DiplomaticRelations` Component. Wait, if I use `Query<&mut DiplomaticRelations>`, the RED phase test would need to spawn an entity with `DiplomaticRelations`.
      ```
    * Let me adjust the RED phase test to use the `DiplomaticRelations` component, to ground it in reality:
      ```rust
      let diplomacy_entity = app.world_mut().spawn(DiplomaticRelations {
          relations: vec![DiplomaticStanding {
              target_id: "player".to_string(),
              standing: 100.0,
              sanctioned: false,
          }],
      }).id();
      ```
6. Implement GREEN phase: Update `process_hyper_resources` to loop over all `DiplomaticRelations` entities and decrease `standing` of all `DiplomaticStanding` by `5.0 * amount`.
7. Refactor phase: extract penalty into `ResourceCurseSettings` resource.
8. Complete Pre commit steps.
9. Verify coverage and submit.
