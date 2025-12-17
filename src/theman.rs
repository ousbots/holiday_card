use bevy::{audio::Volume, prelude::*};
use rand::{Rng, rng};
use std::time::Duration;

use crate::{
    animation::AnimationConfig,
    input::{Direction, InputEvent},
    interaction::{InRange, InteractionEvent, Interactor},
};

#[derive(Component, Clone, Copy, PartialEq)]
enum State {
    Idle,
    Action,
    Walking,
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
pub struct Navigation {
    pub x: f32,
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
    standing_sprite: Handle<Image>,
    standing_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct TheMan;

const WALKING_SPEED: f32 = 30.0;
const WALKING_VOLUME: f32 = 0.85;

const WALKING_TIMER: f32 = 0.45;
const WALKING_TIMER_DELAY: f32 = 0.225;

// NOTE: not sure why the audio width needs to be negative to sound right.
const AUDIO_WIDTH: f32 = -10.0;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_message::<InputEvent>().add_systems(Startup, init).add_systems(
        Update,
        (
            handle_animations,
            handle_audio,
            handle_messages,
            handle_movement,
            handle_interactions,
            handle_navigation_finished,
            idle_action,
        ),
    );
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
                timer.0.set_duration(Duration::from_secs_f32(WALKING_TIMER_DELAY));
            }
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

// Read animation messages and update animation state.
fn handle_messages(
    mut commands: Commands,
    mut events: MessageReader<InputEvent>,
    sprite_assets: Res<SpriteAssets>,
    query: Single<
        (
            Entity,
            &mut AnimationConfig,
            &mut Sprite,
            &mut State,
            &mut Direction,
            &Transform,
        ),
        With<TheMan>,
    >,
) {
    let (entity, mut config, mut sprite, mut state, mut direction, transform) = query.into_inner();
    for event in events.read() {
        match (event.direction, event.target) {
            (None, None) => {
                sprite.image = sprite_assets.standing_sprite.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: sprite_assets.standing_layout.clone(),
                    index: 0,
                });
                sprite.flip_x = *direction == Direction::Left;
                *state = State::Idle;
            }

            (Some(event_direction), None) => match event_direction {
                Direction::Left => {
                    sprite.image = sprite_assets.walking_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.walking_layout.clone(),
                        index: 0,
                    });
                    sprite.flip_x = true;
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    *state = State::Walking;
                    *direction = event_direction;
                }

                Direction::Right => {
                    sprite.image = sprite_assets.walking_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.walking_layout.clone(),
                        index: 0,
                    });
                    sprite.flip_x = false;
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    *state = State::Walking;
                    *direction = event_direction;
                }

                Direction::Up => {
                    sprite.image = sprite_assets.standing_sprite.clone();
                    sprite.texture_atlas = None;
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

                commands.entity(entity).insert(Navigation { x: target.x });

                match event_direction {
                    Direction::Right => {
                        sprite.image = sprite_assets.walking_sprite.clone();
                        sprite.texture_atlas = Some(TextureAtlas {
                            layout: sprite_assets.walking_layout.clone(),
                            index: 0,
                        });
                        sprite.flip_x = false;
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                        *state = State::Walking;
                        *direction = event_direction;
                    }

                    Direction::Left => {
                        sprite.image = sprite_assets.walking_sprite.clone();
                        sprite.texture_atlas = Some(TextureAtlas {
                            layout: sprite_assets.walking_layout.clone(),
                            index: 0,
                        });
                        sprite.flip_x = true;
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
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
                println!("recieved input event with both direction and target data, ignoring!");
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
                    *state = State::Idle;
                    commands.entity(entity).remove::<Navigation>();
                    continue;
                }

                // Walking transformation.
                match *direction {
                    Direction::Left => {
                        transform.translation.x -= WALKING_SPEED * time.delta_secs();
                    }
                    Direction::Right => {
                        transform.translation.x += WALKING_SPEED * time.delta_secs();
                    }
                    Direction::Up => {}
                }
            }

            State::Idle | State::Action => (),
        }
    }
}

// Return the man to the idle state after a Navigation component was removed.
fn handle_navigation_finished(
    sprite_assets: Res<SpriteAssets>,
    mut query: Query<(&mut Sprite, &mut State, &Direction), With<TheMan>>,
    mut removed: RemovedComponents<Navigation>,
) {
    for entity in removed.read() {
        if let Ok((mut sprite, mut state, direction)) = query.get_mut(entity) {
            sprite.image = sprite_assets.standing_sprite.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: sprite_assets.standing_layout.clone(),
                index: 0,
            });
            sprite.flip_x = *direction == Direction::Left;
            *state = State::Action;
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
        walking_sprite: asset_server.load("theman/theman_walking_animation.png"),
        walking_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 9, 1, None, None)),
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
        SpatialListener::new(AUDIO_WIDTH),
        Interactor {
            width: 32.0,
            height: 32.0,
        },
    ));
}
