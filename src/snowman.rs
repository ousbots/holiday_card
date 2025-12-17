use bevy::prelude::*;

#[derive(Component)]
struct Snowman;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init);
}

// Snowman initialization.
fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = asset_server.load("snowman/snowman.png");
    commands.spawn((
        Sprite {
            image: background,
            ..default()
        },
        Transform::from_xyz(-124.0, -53.0, 1.0),
        Snowman,
    ));
}
