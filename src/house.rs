use bevy::prelude::*;

#[derive(Component)]
struct Background;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init);
}

// House initialization.
fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create the house.
    let background = asset_server.load("house.png");
    commands.spawn((
        Sprite {
            image: background,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        Background,
    ));
}
