use bevy::prelude::*;

use crate::animation::AnimationConfig;

#[derive(Component, Clone, Copy, PartialEq)]
enum AnimationState {
    Idle,
    WalkingLeft,
    WalkingRight,
}

#[derive(Message)]
struct AnimationTrigger {
    state: AnimationState,
}

#[derive(Component)]
struct IdleTimer(Timer);

#[derive(Resource)]
struct SpriteAssets {
    walking_sprite: Handle<Image>,
    walking_layout: Handle<TextureAtlasLayout>,
    standing_sprite: Handle<Image>,
    standing_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct TheMan;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_message::<AnimationTrigger>()
        .add_systems(Startup, init)
        .add_systems(Update, (handle_animations, idle_action))
        .add_systems(Update, (handle_keys, trigger_animation::<TheMan>))
        .add_systems(Update, handle_movement);
}

// Loop through all the man's sprites and advance their animation.
fn handle_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &AnimationState)>) {
    for (mut config, mut sprite, state) in &mut query {
        // Idle state only has one frame so skip.
        if *state == AnimationState::Idle {
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

// Handle key input and send animation events.
fn handle_keys(keyboard: Res<ButtonInput<KeyCode>>, mut events: MessageWriter<AnimationTrigger>) {
    // Check for key presses.
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        events.write(AnimationTrigger {
            state: AnimationState::WalkingLeft,
        });
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        events.write(AnimationTrigger {
            state: AnimationState::WalkingRight,
        });
    }

    // Check for key releases.
    if (keyboard.just_released(KeyCode::ArrowLeft) || keyboard.just_released(KeyCode::ArrowRight))
        && !keyboard.pressed(KeyCode::ArrowLeft)
        && !keyboard.pressed(KeyCode::ArrowRight)
    {
        events.write(AnimationTrigger {
            state: AnimationState::Idle,
        });
    }
}

// Move the man based on the current state.
fn handle_movement(time: Res<Time>, mut sprite_position: Query<(&AnimationState, &mut Transform)>) {
    for (state, mut transform) in &mut sprite_position {
        match *state {
            AnimationState::Idle => (),
            AnimationState::WalkingLeft => transform.translation.x -= 110. * time.delta_secs(),
            AnimationState::WalkingRight => transform.translation.x += 110. * time.delta_secs(),
        }
    }
}

// Change the man's direction using the idle timer.
fn idle_action(time: Res<Time>, mut query: Query<(&mut IdleTimer, &mut Sprite, &AnimationState)>) {
    for (mut timer, mut sprite, state) in &mut query {
        if *state == AnimationState::Idle {
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
    // Load the walking sprite sheet.
    let walking_sprite = asset_server.load("man_walking_animation.png");
    let walking_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 9, 1, None, None);
    let walking_layout_handle = texture_layouts.add(walking_layout);

    // Load the idle sprite sheet.
    let standing_sprite = asset_server.load("man_standing.png");
    let standing_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None);
    let standing_layout_handle = texture_layouts.add(standing_layout);

    commands.insert_resource(SpriteAssets {
        walking_sprite: walking_sprite,
        walking_layout: walking_layout_handle,
        standing_sprite: standing_sprite.clone(),
        standing_layout: standing_layout_handle.clone(),
    });

    // Create the man starting in the idle state.
    commands.spawn((
        Sprite {
            image: standing_sprite,
            texture_atlas: Some(TextureAtlas {
                layout: standing_layout_handle,
                index: 0,
            }),
            ..default()
        },
        Transform::from_scale(Vec3::splat(4.0)).with_translation(Vec3::new(-200.0, -50.0, 0.0)),
        TheMan,
        AnimationConfig::new(0, 8, 10),
        AnimationState::Idle,
        IdleTimer(Timer::from_seconds(5.0, TimerMode::Repeating)),
    ));
}

// Read animation messages and update animation state.
fn trigger_animation<S: Component>(
    mut events: MessageReader<AnimationTrigger>,
    sprite_assets: Res<SpriteAssets>,
    query: Single<(&mut AnimationConfig, &mut Sprite, &mut AnimationState), With<S>>,
) {
    let (mut config, mut sprite, mut state) = query.into_inner();
    for event in events.read() {
        let new_state = event.state;

        // Only update if state changed
        if *state != new_state {
            match new_state {
                AnimationState::Idle => {
                    sprite.image = sprite_assets.standing_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.standing_layout.clone(),
                        index: 0,
                    });
                    if *state == AnimationState::WalkingLeft {
                        sprite.flip_x = true;
                    } else {
                        sprite.flip_x = false;
                    }
                }

                AnimationState::WalkingLeft => {
                    sprite.image = sprite_assets.walking_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.walking_layout.clone(),
                        index: 0,
                    });
                    sprite.flip_x = true;
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }

                AnimationState::WalkingRight => {
                    sprite.image = sprite_assets.walking_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.walking_layout.clone(),
                        index: 0,
                    });
                    sprite.flip_x = false;
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }
            }

            *state = new_state;
        }
    }
}
