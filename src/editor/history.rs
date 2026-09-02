use crate::sim::{BodyId, World};

/// A single snapshot in the level editor's undo / redo history.
#[derive(Clone, Debug)]
pub struct EditorSnapshot {
    /// Deep copy of the authoring world (Frame 0*).
    pub world: World,
    /// Selected body IDs at the time of this state.
    pub selected_body_ids: Vec<BodyId>,
    /// Human-readable action description.
    pub description: String,
}

impl EditorSnapshot {
    pub fn new(world: &World, selected_body_ids: Vec<BodyId>, description: impl Into<String>) -> Self {
        Self {
            world: world.clone(),
            selected_body_ids,
            description: description.into(),
        }
    }
}
