use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationConfig,
    interaction::{Highlight, Interactable, InteractionEvent},
};

#[derive(Clone, Component, Copy, PartialEq)]
enum State {
    Off,
    On,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    on_sprite: Handle<Image>,
    on_layout: Handle<TextureAtlasLayout>,
    off_sprite: Handle<Image>,
}

#[derive(Component)]
struct Tree;

const SPRITE_SCALE: f32 = 2.0;
const SPRITE_WIDTH: f32 = 14.;
const SPRITE_HEIGHT: f32 = 16.;

const INTERACTABLE_ID: &str = "tree";

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(
        Update,
        (
            handle_animations,
            handle_highlight,
            handle_highlight_reset,
            handle_interaction,
            handle_interaction_disable_highlight,
        ),
    );
}

// Manage the animation frame timing.
fn handle_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &State), With<Tree>>) {
    let mut rng = rand::rng();

    for (mut config, mut sprite, state) in &mut query {
        // Off state only has one frame so skip.
        if *state == State::Off {
            continue;
        }

        // Track how long the current sprite has been displayed.
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            // Tree sparkles are random.
            let mut new_index = rng.random_range(config.first_index..=config.last_index);
            while new_index == atlas.index {
                new_index = rng.random_range(config.first_index..=config.last_index);
            }
            atlas.index = new_index;
            config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
        }
    }
}

// Apply a pulsing scale effect to highlighted tree.
fn handle_highlight(
    time: Res<Time>,
    query: Query<(&State, &mut Sprite, &mut Transform, &Highlight, &Interactable), (With<Tree>, With<Highlight>)>,
) {
    for (state, mut sprite, mut transform, highlight, interactable) in query {
        if *state == State::Off && interactable.first {
            let pulse = (((time.elapsed_secs() - highlight.elapsed_offset) * 4.).sin() + 1.).mul_add(0.1, 1.);
            sprite.color = Color::srgba(pulse, pulse, pulse, 1.);
            transform.scale = Vec3::splat(SPRITE_SCALE * (((pulse - 1.) / 4.) + 1.));
        } else {
            sprite.color = Color::WHITE;
            transform.scale = Vec3::splat(SPRITE_SCALE);
        }
    }
}

// Reset sprite color when highlight is removed.
fn handle_highlight_reset(
    mut removed: RemovedComponents<Highlight>,
    mut query: Query<(&mut Sprite, &mut Transform), With<Tree>>,
) {
    for entity in removed.read() {
        if let Ok((mut sprite, mut transform)) = query.get_mut(entity) {
            sprite.color = Color::WHITE;
            transform.scale = Vec3::splat(SPRITE_SCALE);
        }
    }
}

// Listen for interaction events and update the state.
fn handle_interaction(
    sprite_assets: Res<SpriteAssets>,
    mut events: MessageReader<InteractionEvent>,
    mut query: Query<(&mut State, &mut Sprite), With<Tree>>,
) {
    for event in events.read() {
        if event.id == INTERACTABLE_ID
            && let Ok((mut state, mut sprite)) = query.single_mut()
        {
            match *state {
                State::Off => {
                    *state = State::On;
                    sprite.image = sprite_assets.on_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.on_layout.clone(),
                        index: 0,
                    });
                }

                State::On => {
                    *state = State::Off;
                    sprite.image = sprite_assets.off_sprite.clone();
                    sprite.texture_atlas = None;
                }
            }
        }
    }
}

fn handle_interaction_disable_highlight(
    mut query: Query<(&mut State, &mut Interactable), (With<Tree>, Changed<State>)>,
) {
    for (state, mut interactable) in &mut query {
        if *state == State::On {
            interactable.first = false;
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
    let sprite = SpriteAssets {
        on_sprite: asset_server.load("tree/tree_animation.png"),
        on_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None)),
        off_sprite: asset_server.load("tree/tree.png"),
    };
    commands.insert_resource(sprite.clone());

    // Create the sprite starting in the off state.
    commands.spawn((
        Sprite {
            image: sprite.off_sprite,
            texture_atlas: None,
            ..default()
        },
        Transform::from_scale(Vec3::splat(SPRITE_SCALE)).with_translation(Vec3::new(-90.0, -62.0, 5.0)),
        Tree,
        AnimationConfig::new(0, 4, 2),
        State::Off,
        Interactable {
            id: INTERACTABLE_ID.to_string(),
            height: SPRITE_HEIGHT * SPRITE_SCALE,
            width: SPRITE_WIDTH * SPRITE_SCALE,
            first: true,
        },
    ));
}
