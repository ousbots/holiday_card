use std::time::Duration;

use bevy::prelude::*;

use crate::{
    animation::AnimationConfig,
    tree::{Presents, Tree},
};

#[derive(Clone, Resource)]
struct SpriteAssets {
    animation_sprite: Handle<Image>,
    animation_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct Santa;

#[derive(Component)]
pub struct Run;

#[derive(Message)]
pub struct AddPresentsEvent;

#[derive(Message)]
pub struct SantasHereEvent;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_message::<AddPresentsEvent>()
        .add_message::<SantasHereEvent>()
        .add_systems(Startup, init)
        .add_systems(Update, (handle_animations, handle_start));
}

// Advance animation frames and states.
fn handle_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut presents_events: MessageWriter<AddPresentsEvent>,
    mut query: Query<(Entity, &mut AnimationConfig, &mut Sprite), (With<Santa>, With<Run>)>,
) {
    for (entity, mut config, mut sprite) in &mut query {
        config.frame_timer.tick(time.delta());

        let mut finished = false;
        let mut add_presents = false;
        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            // Animation plays once and is removed.
            if atlas.index < config.last_index {
                atlas.index += 1;
            } else {
                if atlas.index == 27 {
                    add_presents = true;
                }
                finished = true;
            }

            // Custom frame timing.
            // NOTE: This is instead of copying redundant frames in the spritesheet.
            config.frame_timer = match atlas.index {
                2 => Timer::new(Duration::from_millis(500), TimerMode::Once),
                3 => Timer::new(Duration::from_millis(750), TimerMode::Once),
                7 | 12 => Timer::new(Duration::from_secs(2), TimerMode::Once),
                19..24 => Timer::new(Duration::from_millis(350), TimerMode::Once),
                27 => Timer::new(Duration::from_millis(250), TimerMode::Once),
                _ => AnimationConfig::timer_from_fps(config.fps),
            };
        }

        if finished {
            *sprite = Sprite::default();
            commands.entity(entity).remove::<Run>();
            commands.entity(entity).remove::<Sprite>();
            if add_presents {
                presents_events.write(AddPresentsEvent);
            }
        }
    }
}

fn handle_start(
    mut commands: Commands,
    sprite_assets: Res<SpriteAssets>,
    mut events: MessageReader<SantasHereEvent>,
    mut query: Query<(Entity, &mut AnimationConfig), With<Santa>>,
    tree_query: Query<&Children, With<Tree>>,
    presents_query: Query<Entity, With<Presents>>,
) {
    for _event in events.read() {
        for (entity, mut config) in &mut query {
            let has_presents = tree_query
                .iter()
                .flat_map(|children| children.iter())
                .any(|child| presents_query.contains(child));

            if has_presents {
                *config = AnimationConfig::new(0, 7, 6);
            } else {
                *config = AnimationConfig::new(0, 27, 6);
            }
            commands.entity(entity).insert(Sprite {
                image: sprite_assets.animation_sprite.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_assets.animation_layout.clone(),
                    index: 0,
                }),
                ..default()
            });
            commands.entity(entity).insert(Run);
        }
    }
}

// Initialize the santa animation sprite sheet.
fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let sprites = SpriteAssets {
        animation_sprite: asset_server.load("santa/santa_animation.png"),
        animation_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 28, 1, None, None)),
    };
    commands.insert_resource(sprites);

    commands.spawn((
        Transform::from_translation(Vec3::new(-35.0, -56.0, 10.0)),
        Santa,
        AnimationConfig::new(0, 27, 4),
    ));
}
