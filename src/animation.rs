use bevy::prelude::*;
use std::time::Duration;

use crate::{background, fireplace, interaction, theman};

#[derive(Component)]
pub struct AnimationConfig {
    pub first_index: usize,
    pub last_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_index: first,
            last_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init);
    background::add_systems(app);
    fireplace::add_systems(app);
    interaction::add_systems(app);
    theman::add_systems(app);
}

// Animation initialization.
fn init(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Display help UI in the upper left.
    commands.spawn((
        Text::new("the scene"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}
