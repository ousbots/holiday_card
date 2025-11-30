use bevy::{camera::ScalingMode, prelude::*};
use std::time::Duration;

use crate::{background, fireplace, house, interaction, snow, stereo, theman, tree};

#[derive(Component)]
pub struct AnimationConfig {
    pub first_index: usize,
    pub last_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

const WINDOW_HEIGHT: f32 = 200.0;
const WINDOW_WIDTH: f32 = 400.0;

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
        Timer::new(Duration::from_secs_f32(1.0 / f32::from(fps)), TimerMode::Once)
    }
}

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init);
    background::add_systems(app);
    interaction::add_systems(app);
    house::add_systems(app);
    fireplace::add_systems(app);
    snow::add_systems(app);
    stereo::add_systems(app);
    theman::add_systems(app);
    tree::add_systems(app);
}

// Animation initialization.
fn init(mut commands: Commands) {
    // Create the camera projection.
    let mut ortho = OrthographicProjection::default_2d();
    ortho.scaling_mode = ScalingMode::Fixed {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    };
    let projection = Projection::Orthographic(ortho);

    // Display help UI in the upper left.
    commands.spawn((
        Camera2d,
        projection,
        Text::new("the scene"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}
