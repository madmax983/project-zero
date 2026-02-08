use crate::layer1::needs::Needs;

/// Evaluates the utility of being idle.
///
/// Idle is a low-priority fallback action. Pops should prefer productive
/// activities (work, eating, resting) over standing around.
#[must_use]
pub const fn evaluate_idle(_needs: &Needs) -> f32 {
    0.05
}
