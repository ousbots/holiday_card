use bevy::{audio::Volume, prelude::*};
use rand::{Rng, rng};
use std::time::Duration;

use crate::{
    animation::AnimationConfig,
    chair,
    input::{Direction, InputEvent},
    interaction::{InRange, InteractionEvent, Interactor},
    santa::SantasHereEvent,
};

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum State {
    Idle,
    Action,
    Walking,
    Sitting,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum FootStep {
    Left,
    Right,
}

#[derive(Component)]
struct IdleTimer(Timer);

#[derive(Component)]
struct StepTimer(Timer);

#[derive(Component)]
struct Navigation {
    x: f32,
    action: bool,
}

#[derive(Clone, Resource)]
struct AudioAssets {
    left_steps: Vec<Handle<AudioSource>>,
    right_steps: Vec<Handle<AudioSource>>,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    walking_sprite: Handle<Image>,
    walking_layout: Handle<TextureAtlasLayout>,
    sitting_sprite: Handle<Image>,
    sitting_layout: Handle<TextureAtlasLayout>,
    standing_sprite: Handle<Image>,
    standing_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct TheMan;

const WALKING_SPEED: f32 = 30.0;
const WALKING_VOLUME: f32 = 0.85;
const WALKING_TIMER: f32 = 0.45;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_message::<InputEvent>().add_systems(Startup, init).add_systems(
        Update,
        (
            handle_audio,
            handle_animations,
            handle_animation_state_change.before(handle_animations),
            handle_interactions,
            handle_messages.before(handle_animation_state_change),
            handle_movement,
            handle_idle_action,
            handle_chair_interaction,
        ),
    );
}

// Advance animation frames and states.
fn handle_animations(time: Res<Time>, mut query: Query<(&State, &mut AnimationConfig, &mut Sprite), With<TheMan>>) {
    for (state, mut config, mut sprite) in &mut query {
        // Idle and Action states don't have animations.
        if matches!(*state, State::Idle | State::Action) {
            continue;
        }

        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            match *state {
                // Sitting animation plays once and remains on last frame.
                State::Sitting => {
                    if atlas.index < config.last_index {
                        atlas.index += 1;
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    }
                }

                // Walking animation loops continuously.
                State::Walking => {
                    if atlas.index == config.last_index {
                        atlas.index = config.first_index;
                    } else {
                        atlas.index += 1;
                    }
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }

                State::Idle | State::Action => {}
            }
        }
    }
}

// Handle sprite swapping on state changes.
fn handle_animation_state_change(
    sprite_assets: Res<SpriteAssets>,
    mut query: Query<(&State, &mut Sprite, &mut AnimationConfig, &Direction), (With<TheMan>, Changed<State>)>,
) {
    for (state, mut sprite, mut config, direction) in &mut query {
        match *state {
            State::Idle => {
                sprite.image = sprite_assets.standing_sprite.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: sprite_assets.standing_layout.clone(),
                    index: 0,
                });
                sprite.flip_x = *direction == Direction::Left;
            }

            State::Walking => {
                sprite.image = sprite_assets.walking_sprite.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: sprite_assets.walking_layout.clone(),
                    index: 0,
                });
                sprite.flip_x = *direction == Direction::Left;
                config.first_index = 0;
                config.last_index = 8;
                config.fps = 10;
                config.frame_timer = AnimationConfig::timer_from_fps(10);
            }

            State::Sitting => {
                sprite.image = sprite_assets.sitting_sprite.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: sprite_assets.sitting_layout.clone(),
                    index: 0,
                });
                sprite.flip_x = false;
                config.first_index = 0;
                config.last_index = 4;
                config.fps = 10;
                config.frame_timer = AnimationConfig::timer_from_fps(10);
            }

            State::Action => {
                sprite.image = sprite_assets.standing_sprite.clone();
                sprite.texture_atlas = None;
            }
        }
    }
}

// Runs every frame to tick footstep timer during Walking state.
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
                            commands.spawn((
                                AudioPlayer::new(
                                    audio_assets.left_steps[rng().random_range(0..audio_assets.left_steps.len())]
                                        .clone(),
                                ),
                                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(WALKING_VOLUME)),
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
                                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(WALKING_VOLUME)),
                            ));
                            timer.0.set_duration(Duration::from_secs_f32(WALKING_TIMER));
                            *footstep = FootStep::Left;
                        }
                    }
                }
            }
            _ => {
                timer.0.set_duration(Duration::from_secs_f32(0.225));
            }
        }
    }
}

// Handle chair-specific interactions for sitting/standing.
fn handle_chair_interaction(
    sprite_assets: Res<SpriteAssets>,
    mut events: MessageReader<InteractionEvent>,
    mut santa_events: MessageWriter<SantasHereEvent>,
    mut man_query: Query<(&mut State, &mut Sprite, &mut Transform, &mut AnimationConfig), With<TheMan>>,
) {
    for event in events.read() {
        if event.id != chair::INTERACTABLE_ID {
            continue;
        }

        if let Ok((mut state, mut sprite, mut transform, mut config)) = man_query.single_mut() {
            match *state {
                State::Action => {
                    // Teleport to the chair sitting position.
                    transform.translation.x = 74.0;
                    transform.translation.y = -56.0;
                    transform.translation.z = 4.0;

                    // Switch to the sitting sprite textures and configuration.
                    sprite.image = sprite_assets.sitting_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.sitting_layout.clone(),
                        index: 0,
                    });
                    sprite.flip_x = false;

                    config.first_index = 0;
                    config.last_index = 4;
                    config.fps = 10;
                    config.frame_timer = AnimationConfig::timer_from_fps(10);

                    *state = State::Sitting;
                    santa_events.write(SantasHereEvent);
                }

                State::Sitting => {
                    transform.translation.z = 10.0;

                    // Switch to standing sprite
                    sprite.image = sprite_assets.standing_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.standing_layout.clone(),
                        index: 0,
                    });

                    *state = State::Idle;
                }

                _ => {}
            }
        }
    }
}

// Change the man's direction using the idle timer.
fn handle_idle_action(time: Res<Time>, mut query: Query<(&mut IdleTimer, &mut Sprite, &State), With<TheMan>>) {
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

// Interact with objects when they're in range and the man is in the action state.
fn handle_interactions(
    mut interaction_events: MessageWriter<InteractionEvent>,
    state_query: Query<&State, (With<TheMan>, Changed<State>)>,
    range_query: Query<&InRange>,
) {
    for state in &state_query {
        for in_range in &range_query {
            if *state == State::Action {
                interaction_events.write(InteractionEvent {
                    id: in_range.id.clone(),
                });
            }
        }
    }
}

// Read input messages and update state and direction.
fn handle_messages(
    mut commands: Commands,
    mut events: MessageReader<InputEvent>,
    query: Single<(Entity, &mut State, &mut Direction, &Transform), With<TheMan>>,
) {
    let (entity, mut state, mut direction, transform) = query.into_inner();

    for event in events.read() {
        match (event.direction, event.target) {
            (None, None) => {
                if *state != State::Action && *state != State::Sitting {
                    *state = State::Idle;
                }
            }

            (Some(event_direction), None) => match event_direction {
                Direction::Left | Direction::Right => {
                    *state = State::Walking;
                    *direction = event_direction;
                }

                Direction::Up => {
                    *state = State::Action;
                }
            },

            (None, Some(target)) => {
                let event_direction = if target.x > transform.translation.x {
                    Direction::Right
                } else if target.x < transform.translation.x {
                    Direction::Left
                } else {
                    Direction::Up
                };

                commands.entity(entity).insert(Navigation {
                    x: target.x,
                    action: target.action,
                });

                match event_direction {
                    Direction::Left | Direction::Right => {
                        *state = State::Walking;
                        *direction = event_direction;
                    }

                    Direction::Up => {
                        *state = State::Idle;
                        *direction = Direction::Up;
                    }
                }
            }

            (Some(_), Some(_)) => {
                println!("received input event with both direction and target data, ignoring!");
            }
        }
    }
}

// Move the man based on the current state.
fn handle_movement(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &mut State, &Direction, &mut Transform, Option<&Navigation>), With<TheMan>>,
) {
    for (entity, mut state, direction, mut transform, navigation) in query {
        match *state {
            State::Walking => {
                // Check navigation status.
                if let Some(target) = navigation
                    && ((*direction == Direction::Left && transform.translation.x <= target.x)
                        || (*direction == Direction::Right && transform.translation.x >= target.x))
                {
                    *state = if target.action { State::Action } else { State::Idle };
                    commands.entity(entity).remove::<Navigation>();
                    continue;
                }

                // Walking transformation.
                match *direction {
                    Direction::Left => {
                        transform.translation.x -= WALKING_SPEED * time.delta_secs();
                        transform.translation.x = transform.translation.x.max(-82.0);
                        transform.translation.z = 10.0;
                    }

                    Direction::Right => {
                        transform.translation.x += WALKING_SPEED * time.delta_secs();
                        transform.translation.x = transform.translation.x.min(160.0);
                        transform.translation.z = 10.0;
                    }

                    Direction::Up => {}
                }
            }

            State::Idle | State::Action | State::Sitting => (),
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
        walking_sprite: asset_server.load("theman/theman_walking_animation.png"),
        walking_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 9, 1, None, None)),
        sitting_sprite: asset_server.load("theman/theman_sitting_animation.png"),
        sitting_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None)),
        standing_sprite: asset_server.load("theman/theman_standing.png"),
        standing_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None)),
    };
    commands.insert_resource(sprites.clone());

    // Load the sound effects.
    let mut audio = AudioAssets {
        left_steps: vec![],
        right_steps: vec![],
    };
    audio
        .left_steps
        .push(asset_server.load("theman/left_footstep_indoor_1.ogg"));
    audio
        .left_steps
        .push(asset_server.load("theman/left_footstep_indoor_2.ogg"));
    audio
        .left_steps
        .push(asset_server.load("theman/left_footstep_indoor_3.ogg"));
    audio
        .right_steps
        .push(asset_server.load("theman/right_footstep_indoor_1.ogg"));
    audio
        .right_steps
        .push(asset_server.load("theman/right_footstep_indoor_2.ogg"));
    audio
        .right_steps
        .push(asset_server.load("theman/right_footstep_indoor_3.ogg"));
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
        Transform::from_translation(Vec3::new(-64.0, -56.0, 10.0)),
        TheMan,
        AnimationConfig::new(0, 8, 10),
        State::Idle,
        IdleTimer(Timer::from_seconds(5.0, TimerMode::Repeating)),
        StepTimer(Timer::from_seconds(0.0, TimerMode::Repeating)),
        Direction::Right,
        FootStep::Left,
        // NOTE: not sure why the audio width needs to be negative to sound right.
        SpatialListener::new(-10.0),
        Interactor {
            width: 13.0,
            height: 32.0,
        },
    ));
}
