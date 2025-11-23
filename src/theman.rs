use bevy::{audio::Volume, prelude::*};
use rand::{Rng, rng};
use std::time::Duration;

use crate::animation::AnimationConfig;
use crate::interaction::{InRange, InteractionEvent, Interactor};

#[derive(Component, Clone, Copy, PartialEq)]
enum State {
    Idle,
    Action,
    Walking,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum Direction {
    Left,
    Right,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum FootStep {
    Left,
    Right,
}

#[derive(Message)]
struct Trigger {
    state: State,
    direction: Direction,
}

#[derive(Component)]
struct IdleTimer(Timer);

#[derive(Component)]
struct StepTimer(Timer);

#[derive(Clone, Resource)]
struct AudioAssets {
    left_steps: Vec<Handle<AudioSource>>,
    right_steps: Vec<Handle<AudioSource>>,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    walking_sprite: Handle<Image>,
    walking_layout: Handle<TextureAtlasLayout>,
    standing_sprite: Handle<Image>,
    standing_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct TheMan;

const WALKING_SPEED: f32 = 90.;
const WALKING_VOLUME: f32 = 1.;

const WALKING_TIMER: f32 = 0.45;
const WALKING_TIMER_DELAY: f32 = 0.225;

const AUDIO_WIDTH: f32 = -8.;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_message::<Trigger>()
        .add_systems(Startup, init)
        .add_systems(Update, (handle_animations, idle_action))
        .add_systems(Update, (handle_keys, trigger_animation))
        .add_systems(Update, handle_movement)
        .add_systems(Update, handle_audio);
}

// Loop through all the man's sprites and advance their animation.
fn handle_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &mut State), With<TheMan>>) {
    for (mut config, mut sprite, mut state) in &mut query {
        // Idle state only has one frame so skip.
        if *state == State::Idle {
            continue;
        }

        if *state == State::Action {
            if let Some(atlas) = &sprite.texture_atlas {
                if atlas.index == config.last_index {
                    *state = State::Idle;
                }
            } else {
                *state = State::Idle;
            }
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

// Handle key input and send animation events.
fn handle_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut trigger_events: MessageWriter<Trigger>,
    mut interaction_events: MessageWriter<InteractionEvent>,
    query: Query<&InRange, With<TheMan>>,
) {
    // Check for key presses.
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        trigger_events.write(Trigger {
            state: State::Walking,
            direction: Direction::Left,
        });
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        trigger_events.write(Trigger {
            state: State::Walking,
            direction: Direction::Right,
        });
    } else if keyboard.just_pressed(KeyCode::ArrowUp) {
        trigger_events.write(Trigger {
            state: State::Action,
            direction: Direction::Right,
        });

        // If in range of an interactable, send interaction event
        if let Ok(in_range) = query.single() {
            interaction_events.write(InteractionEvent {
                id: in_range.id.clone(),
            });
        }
    }

    // Check for key releases.
    if keyboard.just_released(KeyCode::ArrowLeft) && !keyboard.pressed(KeyCode::ArrowRight) {
        trigger_events.write(Trigger {
            state: State::Idle,
            direction: Direction::Left,
        });
    }

    if keyboard.just_released(KeyCode::ArrowRight) && !keyboard.pressed(KeyCode::ArrowLeft) {
        trigger_events.write(Trigger {
            state: State::Idle,
            direction: Direction::Right,
        });
    }
}

// Move the man based on the current state.
fn handle_movement(time: Res<Time>, mut sprite_position: Query<(&State, &Direction, &mut Transform), With<TheMan>>) {
    for (state, direction, mut transform) in &mut sprite_position {
        match *state {
            State::Idle | State::Action => (),
            State::Walking => match *direction {
                Direction::Left => {
                    transform.translation.x -= WALKING_SPEED * time.delta_secs();
                }
                Direction::Right => {
                    transform.translation.x += WALKING_SPEED * time.delta_secs();
                }
            },
        }
    }
}

fn handle_audio(
    mut commands: Commands,
    time: Res<Time>,
    audio_assets: Res<AudioAssets>,
    mut query: Query<(&State, &mut StepTimer, &mut FootStep), With<TheMan>>,
) {
    for (state, mut timer, mut footstep) in &mut query {
        match *state {
            State::Walking => {
                timer.0.tick(time.delta());
                if timer.0.just_finished() {
                    match *footstep {
                        FootStep::Left => {
                            // let audio = [audio_assets.left_step_indoor_1, audio_assets.left_step_indoor_2, audio_assets.left_step_indoor_3].choose(rng())
                            commands.spawn((
                                AudioPlayer::new(
                                    audio_assets.left_steps[rng().random_range(0..audio_assets.left_steps.len())]
                                        .clone(),
                                ),
                                PlaybackSettings::ONCE.with_volume(Volume::Linear(WALKING_VOLUME)),
                            ));
                            timer.0.set_duration(Duration::from_secs_f32(WALKING_TIMER));
                            *footstep = FootStep::Right;
                        }
                        FootStep::Right => {
                            commands.spawn((
                                AudioPlayer::new(
                                    audio_assets.right_steps[rng().random_range(0..audio_assets.right_steps.len())]
                                        .clone(),
                                ),
                                PlaybackSettings::ONCE.with_volume(Volume::Linear(WALKING_VOLUME)),
                            ));
                            timer.0.set_duration(Duration::from_secs_f32(WALKING_TIMER));
                            *footstep = FootStep::Left;
                        }
                    }
                }
            }
            _ => {
                timer.0.set_duration(Duration::from_secs_f32(WALKING_TIMER_DELAY));
            }
        }
    }
}

// Change the man's direction using the idle timer.
fn idle_action(time: Res<Time>, mut query: Query<(&mut IdleTimer, &mut Sprite, &State), With<TheMan>>) {
    for (mut timer, mut sprite, state) in &mut query {
        if *state == State::Idle {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                sprite.flip_x = !sprite.flip_x;
            }
        } else {
            timer.0.reset();
        }
    }
}

// Initialize the man.
fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load the sprite sheets.
    let sprites = SpriteAssets {
        walking_sprite: asset_server.load("man_walking_animation.png"),
        walking_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 9, 1, None, None)),
        standing_sprite: asset_server.load("man_standing.png"),
        standing_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None)),
    };
    commands.insert_resource(sprites.clone());

    // Load the sound effects.
    let mut audio = AudioAssets {
        left_steps: vec![],
        right_steps: vec![],
    };
    audio.left_steps.push(asset_server.load("left_footstep_indoor_1.ogg"));
    audio.left_steps.push(asset_server.load("left_footstep_indoor_2.ogg"));
    audio.left_steps.push(asset_server.load("left_footstep_indoor_3.ogg"));
    audio.right_steps.push(asset_server.load("right_footstep_indoor_1.ogg"));
    audio.right_steps.push(asset_server.load("right_footstep_indoor_2.ogg"));
    audio.right_steps.push(asset_server.load("right_footstep_indoor_3.ogg"));
    commands.insert_resource(audio);

    // Create the man starting in the idle state.
    commands.spawn((
        Sprite {
            image: sprites.standing_sprite,
            texture_atlas: Some(TextureAtlas {
                layout: sprites.standing_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_scale(Vec3::splat(4.0)).with_translation(Vec3::new(-200.0, -55.0, 10.0)),
        TheMan,
        AnimationConfig::new(0, 8, 10),
        State::Idle,
        IdleTimer(Timer::from_seconds(5.0, TimerMode::Repeating)),
        StepTimer(Timer::from_seconds(0.0, TimerMode::Repeating)),
        Direction::Right,
        FootStep::Left,
        SpatialListener::new(AUDIO_WIDTH),
        Interactor {
            width: 32.0 * 4.0, // Sprite size (32) * scale (4)
            height: 32.0 * 4.0,
        },
    ));
}

// Read animation messages and update animation state.
fn trigger_animation(
    mut events: MessageReader<Trigger>,
    sprite_assets: Res<SpriteAssets>,
    query: Single<(&mut AnimationConfig, &mut Sprite, &mut State, &mut Direction), With<TheMan>>,
) {
    let (mut config, mut sprite, mut state, mut direction) = query.into_inner();
    for event in events.read() {
        // Only update if state changed
        if *state != event.state || *direction != event.direction {
            match event.state {
                State::Idle => {
                    sprite.image = sprite_assets.standing_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.standing_layout.clone(),
                        index: 0,
                    });
                    sprite.flip_x = *direction == Direction::Left;
                }

                State::Action => {
                    sprite.image = sprite_assets.standing_sprite.clone();
                    sprite.texture_atlas = None;
                }

                State::Walking => match event.direction {
                    Direction::Left => {
                        sprite.image = sprite_assets.walking_sprite.clone();
                        sprite.texture_atlas = Some(TextureAtlas {
                            layout: sprite_assets.walking_layout.clone(),
                            index: 0,
                        });
                        sprite.flip_x = true;
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    }

                    Direction::Right => {
                        sprite.image = sprite_assets.walking_sprite.clone();
                        sprite.texture_atlas = Some(TextureAtlas {
                            layout: sprite_assets.walking_layout.clone(),
                            index: 0,
                        });
                        sprite.flip_x = false;
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    }
                },
            }

            *state = event.state;
            *direction = event.direction;
        }
    }
}
