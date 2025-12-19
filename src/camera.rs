use bevy::{camera::ScalingMode, prelude::*};
use bevy_light_2d::prelude::*;

const WINDOW_HEIGHT: f32 = 150.0;
const WINDOW_WIDTH: f32 = 300.0;

const AMBIENT_BRIGHTNESS: f32 = 0.035;

// Add the camera systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init);
}

// Camera initialization.
fn init(mut commands: Commands) {
    // Create the camera projection.
    let mut ortho = OrthographicProjection::default_2d();
    ortho.scaling_mode = ScalingMode::Fixed {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    };
    let projection = Projection::Orthographic(ortho);

    commands.spawn((
        Camera2d,
        projection,
        Light2d {
            ambient_light: AmbientLight2d {
                brightness: AMBIENT_BRIGHTNESS,
                ..default()
            },
        },
    ));

    // Display help UI in the upper right.
    commands.spawn((
        Text::new("move: left/right - interact: up"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("or click to move and interact"),
        Node {
            position_type: PositionType::Absolute,
            top: px(35),
            right: px(24),
            ..default()
        },
    ));
}
