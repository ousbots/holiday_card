use bevy::prelude::*;
use bevy_light_2d::prelude::*;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct SnowMovement {
    timer: Timer,
    rise: f32,
    progress: f32,
}

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(Update, handle_snow);
}

// Handle the snow rising over time.
fn handle_snow(time: Res<Time>, mut commands: Commands, mut query: Query<(Entity, &mut SnowMovement, &mut Transform)>) {
    for (entity, mut snow, mut transform) in &mut query {
        snow.timer.tick(time.delta());

        let progress = snow.timer.fraction();
        transform.translation.y += (progress - snow.progress) * snow.rise;
        snow.progress = progress;

        if snow.timer.just_finished() {
            commands.entity(entity).remove::<SnowMovement>();
        }
    }
}

// Background initialization.
fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Background.
    let background = asset_server.load("background/background.png");
    commands.spawn((
        Sprite {
            image: background,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Background,
    ));

    // Snow on the ground, z = 1.5 to be in front of the falling snow.
    let snow = asset_server.load("background/snow.png");
    commands.spawn((
        Sprite {
            image: snow,
            ..default()
        },
        Transform::from_xyz(0.0, -75.0, 1.5),
        Background,
        SnowMovement {
            timer: Timer::from_seconds(60.0 * 5.0, TimerMode::Once),
            rise: 15.0,
            progress: 0.0,
        },
    ));

    // Moonlight.
    commands.spawn((
        SpotLight2d {
            color: Color::srgba(1.0, 1.0, 1.0, 1.0),
            intensity: 0.4,
            radius: 200.0,
            direction: 135.0,
            inner_angle: 40.0,
            outer_angle: 60.0,
            source_width: 1.0,
            cast_shadows: true,
            ..default()
        },
        Transform::from_xyz(-160.0, 140.0, 2.0),
    ));
}
