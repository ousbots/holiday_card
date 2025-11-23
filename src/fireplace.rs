use bevy::{audio::Volume, prelude::*};
use rand::Rng;

use crate::animation::AnimationConfig;

#[derive(Component, Clone, Copy, PartialEq)]
enum AnimationState {
    Off,
    Starting,
    Running,
}

#[derive(Resource)]
struct SpriteAssets {
    running_sprite: Handle<Image>,
    running_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct Fireplace;

const RUNNING_VOLUME: f32 = 1.;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(Update, handle_animations);
}

// Loop through all the sprites and advance their animation.
fn handle_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &AnimationState)>) {
    let mut rng = rand::rng();

    for (mut config, mut sprite, state) in &mut query {
        // Off state only has one frame so skip.
        if *state == AnimationState::Off {
            continue;
        }

        // Track how long the current sprite has been displayed.
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            // Fires are random.
            let mut new_index = rng.random_range(config.first_index..=config.last_index);
            while new_index == atlas.index {
                new_index = rng.random_range(config.first_index..=config.last_index);
            }
            atlas.index = new_index;
            config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
        }
    }
}

// Animation initialization.
fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load the running sprite sheet.
    let running_sprite = asset_server.load("fireplace_animation.png");
    let running_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None);
    let running_layout_handle = texture_layouts.add(running_layout);

    commands.insert_resource(SpriteAssets {
        running_sprite: running_sprite.clone(),
        running_layout: running_layout_handle.clone(),
    });

    // Create the sprite starting in the running state.
    commands.spawn((
        Sprite {
            image: running_sprite,
            texture_atlas: Some(TextureAtlas {
                layout: running_layout_handle,
                index: 0,
            }),
            ..default()
        },
        Transform::from_scale(Vec3::splat(7.0)).with_translation(Vec3::new(0.0, 0.0, 1.0)),
        Fireplace,
        AnimationConfig::new(0, 4, 6),
        AnimationState::Running,
        AudioPlayer::new(asset_server.load("fire.ogg")),
        PlaybackSettings::LOOP
            .with_spatial(true)
            .with_volume(Volume::Linear(RUNNING_VOLUME)),
    ));
}
