use bevy::{audio::Volume, prelude::*};

use crate::{
    animation::AnimationConfig,
    interaction::{Highlight, Interactable, InteractionEvent},
};

#[derive(Clone, Component, Copy, PartialEq)]
enum State {
    Off,
    Running,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    running_sprite: Handle<Image>,
    running_layout: Handle<TextureAtlasLayout>,
    off_sprite: Handle<Image>,
}

#[derive(Component)]
struct Stereo;

const RUNNING_VOLUME: f32 = 0.9;

const SPRITE_SCALE: f32 = 2.0;
const SPRITE_WIDTH: f32 = 20.;
const SPRITE_HEIGHT: f32 = 16.;

const INTERACTABLE_ID: &str = "stereo";

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
            handle_sound,
        ),
    );
}

// Manage the animation frame timing.
fn handle_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &State), With<Stereo>>) {
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
            // On last frame, reset to the first, otherwise advance.
            if atlas.index == config.last_index {
                atlas.index = config.first_index;
            } else {
                atlas.index += 1;
            }
            config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
        }
    }
}

// Apply a pulsing scale effect to highlighted stereo.
fn handle_highlight(
    time: Res<Time>,
    query: Query<(&State, &mut Sprite, &mut Transform, &Highlight, &Interactable), (With<Stereo>, With<Highlight>)>,
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
    mut query: Query<(&mut Sprite, &mut Transform), With<Stereo>>,
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
    mut query: Query<(&mut State, &mut Sprite), With<Stereo>>,
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

                State::Running => {
                    *state = State::Off;
                    sprite.image = sprite_assets.off_sprite.clone();
                    sprite.texture_atlas = None;
                }
            }
        }
    }
}

fn handle_interaction_disable_highlight(
    mut query: Query<(&mut State, &mut Interactable), (With<Stereo>, Changed<State>)>,
) {
    for (state, mut interactable) in &mut query {
        if *state == State::Running {
            interactable.first = false;
        }
    }
}

// Control audio playback based on stereo state
fn handle_sound(query: Query<(&State, &mut SpatialAudioSink), (With<Stereo>, Changed<State>)>) {
    for (state, audio_sink) in &query {
        match *state {
            // Start the stereo sound effect if it isn't already running.
            State::Running => {
                audio_sink.play();
            }

            // Remove any existing sound effects.
            State::Off => {
                audio_sink.pause();
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
        running_sprite: asset_server.load("stereo/stereo_animation.png"),
        running_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None)),
        off_sprite: asset_server.load("stereo/stereo.png"),
    };
    commands.insert_resource(sprite.clone());

    // Create the sprite starting in the off state.
    commands.spawn((
        Sprite {
            image: sprite.off_sprite,
            texture_atlas: None,
            ..default()
        },
        Transform::from_scale(Vec3::splat(SPRITE_SCALE)).with_translation(Vec3::new(90.0, -62.0, 5.0)),
        Stereo,
        AnimationConfig::new(0, 4, 4),
        State::Off,
        AudioPlayer::new(asset_server.load("stereo/merry_little_christmas.ogg")),
        PlaybackSettings::LOOP
            .with_spatial(true)
            .with_volume(Volume::Linear(RUNNING_VOLUME))
            .paused(),
        Interactable {
            id: INTERACTABLE_ID.to_string(),
            height: SPRITE_HEIGHT * SPRITE_SCALE,
            width: SPRITE_WIDTH * SPRITE_SCALE,
            first: true,
        },
    ));
}
