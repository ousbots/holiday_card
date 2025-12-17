use bevy::prelude::*;
use bevy_light_2d::prelude::*;

#[derive(Component)]
struct Background;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init);
}

// House initialization.
fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create the house.
    let background = asset_server.load("house/house.png");
    commands.spawn((
        Sprite {
            image: background,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
        Background,
    ));

    // Create three (floor is ignored) rectangle occluders to block light from crossing the house boundaries.
    commands.spawn((
        LightOccluder2d {
            shape: LightOccluder2dShape::Rectangle {
                half_size: Vec2::new(133.0, 2.0),
            },
        },
        Transform::from_xyz(40.0, 10.0, 2.0),
    ));

    commands.spawn((
        LightOccluder2d {
            shape: LightOccluder2dShape::Rectangle {
                half_size: Vec2::new(4.0, 45.0),
            },
        },
        Transform::from_xyz(-90.0, -35.0, 2.0),
    ));

    // Build a diagonal from horizontal pieces due to a bug where LightOccluder2d ignores transformations.
    let x_offset: f32 = -94.0;
    let y_offset: f32 = 12.0;
    let slope: f32 = 1.28;
    for point in 0..92i16 {
        let x = f32::from(point).mul_add(slope, x_offset);
        let y = (f32::from(point) / slope) + y_offset;

        commands.spawn((
            LightOccluder2d {
                shape: LightOccluder2dShape::Rectangle {
                    half_size: Vec2::new(1.0, 1.0),
                },
            },
            Transform::from_xyz(x, y, 2.0),
        ));
    }
}
