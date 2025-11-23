use bevy::prelude::*;

// Add to entities that can initiate interactions.
#[derive(Component)]
pub struct Interactor {
    pub width: f32,
    pub height: f32,
}

// Add to entities that can be interacted with.
#[derive(Component)]
pub struct Interactable {
    pub width: f32,
    pub height: f32,
    pub id: String,
}

// Added to Interactor entities when they're in range of an Interactable.
#[derive(Component)]
pub struct InRange {
    pub id: String,
}

// Message sent when an interaction is triggered.
#[derive(Message)]
pub struct InteractionEvent {
    pub id: String,
}

// Add the interaction systems.
pub fn add_systems(app: &mut App) {
    app.add_message::<InteractionEvent>()
        .add_systems(Update, detect_overlaps);
}

// Detects AABB overlaps between Interactors and Interactables.
fn detect_overlaps(
    mut commands: Commands,
    interactors: Query<(Entity, &Transform, &Interactor)>,
    interactables: Query<(&Transform, &Interactable)>,
    in_range: Query<(Entity, &InRange)>,
) {
    for (interactor_entity, interactor_transform, interactor) in &interactors {
        let mut found_overlap = None;

        // Check against all interactables.
        for (interactable_transform, interactable) in &interactables {
            if aabb_overlap(
                interactor_transform.translation.truncate(),
                interactor.width,
                interactor.height,
                interactable_transform.translation.truncate(),
                interactable.width,
                interactable.height,
            ) {
                found_overlap = Some(interactable.id.clone());
                break;
            }
        }

        // Update InRange component based on overlap.
        let currently_in_range = in_range
            .iter()
            .find(|(e, _)| *e == interactor_entity)
            .map(|(_, r)| r.id.clone());

        match (currently_in_range, found_overlap) {
            // New entity entered in-range.
            (None, Some(id)) => {
                commands.entity(interactor_entity).insert(InRange { id: id });
            }

            // Entity in-range changed.
            (Some(current_id), Some(new_id)) if current_id != new_id => {
                commands.entity(interactor_entity).insert(InRange { id: new_id });
            }

            // Entity left in-range.
            (Some(_), None) => {
                commands.entity(interactor_entity).remove::<InRange>();
            }

            _ => {}
        }
    }
}

// Simple AABB (Axis-Aligned Bounding Box) overlap detection.
fn aabb_overlap(pos_1: Vec2, width_1: f32, height_1: f32, pos_2: Vec2, width_2: f32, height_2: f32) -> bool {
    let half_width_1 = width_1 / 2.0;
    let half_height_1 = height_1 / 2.0;
    let half_width_2 = width_2 / 2.0;
    let half_height_2 = height_2 / 2.0;

    let left_1 = pos_1.x - half_width_1;
    let right_1 = pos_1.x + half_width_1;
    let top_1 = pos_1.y + half_height_1;
    let bottom_1 = pos_1.y - half_height_1;

    let left_2 = pos_2.x - half_width_2;
    let right_2 = pos_2.x + half_width_2;
    let top_2 = pos_2.y + half_height_2;
    let bottom_2 = pos_2.y - half_height_2;

    !(right_1 < left_2 || left_1 > right_2 || top_1 < bottom_2 || bottom_1 > top_2)
}
