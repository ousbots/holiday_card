use bevy::{audio::Volume, prelude::*};
use bevy_light_2d::prelude::*;
use rand::Rng;

use crate::{
    flickering_light::FlickeringLight,
    interaction::{Interactable, InteractionEvent, State},
};

#[derive(Clone, Resource)]
struct AudioAssets {
    on: Handle<AudioSource>,
    off: Handle<AudioSource>,
}

#[derive(Component)]
struct AtticLight;

const INTERACTABLE_ID: &str = "attic-light";

// Sprite parameters.
const SPRITE_WIDTH: f32 = 2.0;
const SPRITE_HEIGHT: f32 = 16.0;

const SWITCH_VOLUME: f32 = 0.40;

// Light effect parameters.
const LIGHT_RADIUS: f32 = 160.0;
const LIGHT_COLORS: [Color; 3] = [
    Color::srgb(1.0, 0.6, 0.2),
    Color::srgb(1.0, 0.7, 0.1),
    Color::srgb(1.0, 0.5, 0.3),
];

const INTENSITY_OCTAVES: u32 = 4;
const COLOR_OCTAVES: u32 = 5;

const INTENSITY_FREQ: f32 = 2.0;
const INTENSITY_MIN: f32 = 0.4;
const INTENSITY_AMPLITUDE: f32 = 0.2;

const COLOR_FREQ: f32 = 100.0;
const COLOR_TEMPERATURE: f32 = 0.5;
const COLOR_SEED_OFFSET: f32 = 100.0;

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init).add_systems(
        Update,
        (
            handle_interaction,
            handle_light.in_set(crate::flickering_light::LightInsertionSet),
        ),
    );
}

// Listen for interaction events and update the state.
fn handle_interaction(mut events: MessageReader<InteractionEvent>, mut query: Query<&mut State, With<AtticLight>>) {
    for event in events.read() {
        if event.id == INTERACTABLE_ID
            && let Ok(mut state) = query.single_mut()
        {
            match *state {
                State::Off => {
                    *state = State::On;
                }

                State::On => {
                    *state = State::Off;
                }
            }
        }
    }
}

// Add or remove flickering light based on the fireplace state.
fn handle_light(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    parent_query: Query<(&Children, &State), (With<AtticLight>, Changed<State>)>,
    mut light_query: Query<(Entity, &mut PointLight2d)>,
) {
    let mut rng = rand::rng();

    // Find the child light entity.
    for (children, state) in &parent_query {
        for child in children.iter() {
            if let Ok((entity, mut light)) = light_query.get_mut(child) {
                match *state {
                    State::On => {
                        commands.spawn((
                            AudioPlayer::new(audio_assets.on.clone()),
                            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(SWITCH_VOLUME)),
                        ));
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
                        commands.spawn((
                            AudioPlayer::new(audio_assets.off.clone()),
                            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(SWITCH_VOLUME)),
                        ));
                        commands.entity(entity).remove::<FlickeringLight>();
                        light.intensity = 0.0;
                    }
                }
            }
        }
    }
}

// Attic light initialization.
fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio = AudioAssets {
        on: asset_server.load("house/light_switch_on.ogg"),
        off: asset_server.load("house/light_switch_off.ogg"),
    };
    commands.insert_resource(audio);

    // Parent position is the hidden switch.
    let parent = commands
        .spawn((
            AtticLight,
            State::Off,
            Sprite::default(),
            Transform::from_xyz(149.0, -54.0, 5.0),
            Interactable {
                id: INTERACTABLE_ID.to_string(),
                height: SPRITE_HEIGHT,
                width: SPRITE_WIDTH,
                ..default()
            },
        ))
        .id();

    // Spawn light, Local offset (-21, 114, 0) → Global position (128, 60, 5)
    let light = commands
        .spawn((
            Transform::from_xyz(-21.0, 114.0, 0.0),
            PointLight2d {
                color: LIGHT_COLORS[0],
                intensity: 0.0,
                radius: LIGHT_RADIUS,
                cast_shadows: true,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(light);
}
