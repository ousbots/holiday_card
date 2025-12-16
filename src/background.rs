use bevy::prelude::*;
use bevy_light_2d::prelude::*;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Snow {
    timer: Timer,
    rise: u32,
}

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(Update, handle_snow);
}

// Handle the snow rising over time.
fn handle_snow(time: Res<Time>, mut query: Query<(&mut Snow, &mut Transform), With<Background>>) {
    for (mut snow, mut transform) in &mut query {
        if snow.rise > 0 {
            snow.timer.tick(time.delta());

            if snow.timer.just_finished() {
                transform.translation.y += 1.0;
                snow.rise -= 1;
            }
        }
    }
}

// Background initialization.
fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Background.
    let background = asset_server.load("background.png");
    commands.spawn((
        Sprite {
            image: background,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Background,
    ));

    // Snow on the ground.
    let snow = asset_server.load("snow.png");
    commands.spawn((
        Sprite {
            image: snow,
            ..default()
        },
        Transform::from_xyz(0.0, -75.0, 0.5),
        Background,
        Snow {
            timer: Timer::from_seconds(60.0, TimerMode::Repeating),
            rise: 15,
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
