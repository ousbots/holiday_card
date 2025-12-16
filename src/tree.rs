use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationConfig,
    flickering_light::FlickeringLight,
    interaction::{Interactable, InteractionEvent, State},
};

#[derive(Clone, Resource)]
struct SpriteAssets {
    on_sprite: Handle<Image>,
    on_layout: Handle<TextureAtlasLayout>,
    off_sprite: Handle<Image>,
}

#[derive(Component)]
struct Tree;

const INTERACTABLE_ID: &str = "tree";

const SPRITE_WIDTH: f32 = 14.;
const SPRITE_HEIGHT: f32 = 16.;

const LIGHT_RADIUS: f32 = 50.0;
const LIGHT_COLORS: [Color; 4] = [
    Color::srgb(0.2, 0.2, 0.8),
    Color::srgb(0.2, 0.8, 0.2),
    Color::srgb(0.8, 0.2, 0.2),
    Color::srgb(0.8, 0.8, 0.8),
];

// Light effect noise parameters.
const INTENSITY_OCTAVES: u32 = 3;
const COLOR_OCTAVES: u32 = 2;

const INTENSITY_FREQ: f32 = 1.0;
const INTENSITY_MIN: f32 = 0.5;
const INTENSITY_AMPLITUDE: f32 = 0.2;

const COLOR_FREQ: f32 = 1.0;
const COLOR_TEMPERATURE: f32 = 0.5;
const COLOR_SEED_OFFSET: f32 = 100.0;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(
        Update,
        (
            handle_animations,
            handle_interaction,
            handle_light.in_set(crate::flickering_light::LightInsertionSet),
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

// Listen for interaction events and update the tree state.
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

// Add or remove a flickering light based on the tree state.
fn handle_light(
    mut commands: Commands,
    mut query: Query<(Entity, &State, &mut PointLight2d), (With<Tree>, Changed<State>)>,
) {
    let mut rng = rand::rng();

    for (entity, state, mut light) in &mut query {
        match *state {
            State::On => {
                commands.entity(entity).insert(FlickeringLight {
                    seed: rng.random_range(0.0..1000.0),
                    intensity_amplitude: INTENSITY_AMPLITUDE,
                    intensity_frequency: INTENSITY_FREQ,
                    intensity_min: INTENSITY_MIN,
                    intensity_octaves: INTENSITY_OCTAVES,
                    color_frequency: COLOR_FREQ,
                    color_octaves: COLOR_OCTAVES,
                    color_seed_offset: COLOR_SEED_OFFSET,
                    color_temperature: COLOR_TEMPERATURE,
                    colors: LIGHT_COLORS.to_vec(),
                    time_offset: rng.random_range(0.0..100.0),
                });
            }
            State::Off => {
                commands.entity(entity).remove::<FlickeringLight>();
                light.intensity = 0.0;
            }
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
        on_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None)),
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
        Transform::from_translation(Vec3::new(4.0, -38.0, 5.0)),
        Tree,
        AnimationConfig::new(0, 4, 2),
        State::Off,
        Interactable {
            id: INTERACTABLE_ID.to_string(),
            height: SPRITE_HEIGHT,
            width: SPRITE_WIDTH,
            ..default()
        },
        PointLight2d {
            color: LIGHT_COLORS[0],
            intensity: 0.0,
            radius: LIGHT_RADIUS,
            cast_shadows: true,
            ..default()
        },
    ));
}
