use bevy::{audio::Volume, prelude::*};
use bevy_light_2d::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationConfig,
    flickering_light::FlickeringLight,
    interaction::{Interactable, InteractionEvent, State},
};

#[derive(Clone, Resource)]
struct SpriteAssets {
    running_sprite: Handle<Image>,
    running_layout: Handle<TextureAtlasLayout>,
    off_sprite: Handle<Image>,
}

#[derive(Component)]
struct Fireplace;

const INTERACTABLE_ID: &str = "fireplace";

// Light effect colors.
const LIGHT_COLORS: [Color; 3] = [
    Color::srgb(1.0, 0.6, 0.2),
    Color::srgb(1.0, 0.62, 0.18),
    Color::srgb(1.0, 0.58, 0.22),
];

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(
        Update,
        (
            handle_animations,
            handle_interaction,
            handle_sound,
            handle_light.in_set(crate::flickering_light::LightInsertionSet),
        ),
    );
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
                    *state = State::On;
                    sprite.image = sprite_assets.running_sprite.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: sprite_assets.running_layout.clone(),
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

// Control audio playback based on fireplace state
fn handle_sound(query: Query<(&State, &mut SpatialAudioSink), (With<Fireplace>, Changed<State>)>) {
    for (state, audio_sink) in &query {
        match *state {
            // Start the fireplace sound effect if it isn't already running.
            State::On => {
                audio_sink.play();
            }

            // Remove any existing sound effects.
            State::Off => {
                audio_sink.pause();
            }
        }
    }
}

// Add or remove flickering light based on the fireplace state.
fn handle_light(
    mut commands: Commands,
    mut query: Query<(Entity, &State, &mut PointLight2d), (With<Fireplace>, Changed<State>)>,
) {
    let mut rng = rand::rng();

    for (entity, state, mut light) in &mut query {
        match *state {
            State::On => {
                commands.entity(entity).insert(FlickeringLight {
                    seed: rng.random_range(0.0..1000.0),
                    intensity_amplitude: 0.4,
                    intensity_frequency: 2.0,
                    intensity_min: 0.6,
                    intensity_octaves: 4,
                    color_frequency: 1.0,
                    color_octaves: 2,
                    color_seed_offset: 100.0,
                    color_temperature: 0.2,
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
        running_sprite: asset_server.load("fireplace/fireplace_animation.png"),
        running_layout: texture_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(64, 78), 5, 1, None, None)),
        off_sprite: asset_server.load("fireplace/fireplace.png"),
    };
    commands.insert_resource(sprite.clone());

    // Create the sprite starting in the off state.
    commands.spawn((
        Sprite {
            image: sprite.off_sprite,
            texture_atlas: None,
            ..default()
        },
        Transform::from_translation(Vec3::new(118.0, -31.0, 5.0)),
        Fireplace,
        AnimationConfig::new(0, 4, 6),
        State::Off,
        AudioPlayer::new(asset_server.load("fireplace/fire.ogg")),
        PlaybackSettings::LOOP
            .with_spatial(true)
            .with_volume(Volume::Linear(0.75))
            .paused(),
        Interactable {
            id: INTERACTABLE_ID.to_string(),
            height: 78.0,
            width: 48.0,
            ..default()
        },
        PointLight2d {
            color: LIGHT_COLORS[0],
            intensity: 0.0,
            radius: 180.0,
            cast_shadows: true,
            ..default()
        },
    ));
}
