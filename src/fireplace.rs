use bevy::{audio::Volume, prelude::*};
use rand::Rng;

use crate::animation::AnimationConfig;
use crate::interaction::{Interactable, InteractionEvent};

#[derive(Clone, Component, Copy, PartialEq)]
enum State {
    Off,
    Starting,
    Running,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    running_sprite: Handle<Image>,
    running_layout: Handle<TextureAtlasLayout>,
    off_sprite: Handle<Image>,
}

#[derive(Component)]
struct Fireplace;

const RUNNING_VOLUME: f32 = 1.;
const SPRITE_SCALE: f32 = 7.;
const SPRITE_WIDTH: f32 = 16.;
const SPRITE_HEIGHT: f32 = 16.;

const INTERACTABLE_ID: &str = "fireplace";

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init)
        .add_systems(Update, (handle_animations, handle_interaction, handle_sound));
}

// Manage the animation frame timing.
fn handle_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &State), With<Fireplace>>) {
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

// Listen for interaction events and update the state.
fn handle_interaction(
    sprite_assets: Res<SpriteAssets>,
    mut events: MessageReader<InteractionEvent>,
    mut query: Query<(&mut State, &mut Sprite), With<Fireplace>>,
) {
    for event in events.read() {
        if event.id == INTERACTABLE_ID
            && let Ok((mut state, mut sprite)) = query.single_mut()
        {
            match *state {
                State::Off => {
                    *state = State::Running;
                    sprite.image = sprite_assets.running_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.running_layout.clone(),
                        index: 0,
                    });
                }

                State::Running | State::Starting => {
                    *state = State::Off;
                    sprite.image = sprite_assets.off_sprite.clone();
                    sprite.texture_atlas = None;
                }
            }
        }
    }
}

// Control audio playback based on fireplace state
fn handle_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &State, Option<&AudioSink>), (With<Fireplace>, Changed<State>)>,
) {
    for (entity, state, audio_sink) in &query {
        match *state {
            // Start the fireplace sound effect if it isn't already running.
            State::Running => {
                if audio_sink.is_none() {
                    commands.entity(entity).insert((
                        AudioPlayer::new(asset_server.load("fire.ogg")),
                        PlaybackSettings::LOOP
                            .with_spatial(true)
                            .with_volume(Volume::Linear(RUNNING_VOLUME)),
                    ));
                }
            }

            // Remove any existing sound effects.
            // BUG: this doesn't stop the sound.
            State::Off => {
                if let Some(sink) = audio_sink {
                    sink.stop();
                }
                commands
                    .entity(entity)
                    .remove::<AudioPlayer>()
                    .remove::<PlaybackSettings>();
            }

            // TODO: add sound effect for the starting state.
            State::Starting => {}
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
        running_sprite: asset_server.load("fireplace_animation.png"),
        running_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None)),
        off_sprite: asset_server.load("fireplace.png"),
    };
    commands.insert_resource(sprite.clone());

    // Create the sprite starting in the off state.
    commands.spawn((
        Sprite {
            image: sprite.off_sprite,
            texture_atlas: None,
            ..default()
        },
        Transform::from_scale(Vec3::splat(SPRITE_SCALE)).with_translation(Vec3::new(0.0, 0.0, 1.0)),
        Fireplace,
        AnimationConfig::new(0, 4, 6),
        State::Off,
        Interactable {
            width: SPRITE_WIDTH * SPRITE_SCALE,
            height: SPRITE_HEIGHT * SPRITE_SCALE,
            id: INTERACTABLE_ID.to_string(),
        },
    ));
}
