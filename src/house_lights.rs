use bevy::{audio::Volume, prelude::*};
use bevy_light_2d::prelude::*;
use rand::Rng;

use crate::{
    flickering_light::FlickeringLight,
    interaction::{InRange, Interactable, InteractionEvent, State},
};

#[derive(Clone, Resource)]
struct AudioAssets {
    on: Handle<AudioSource>,
    off: Handle<AudioSource>,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    switch_on: Handle<Image>,
    switch_off: Handle<Image>,
    xmas_light_red: Handle<Image>,
    xmas_light_yellow: Handle<Image>,
    xmas_light_green: Handle<Image>,
}

#[derive(Component)]
struct Switch;

#[derive(Component)]
struct AtticLight;

#[derive(Clone, Copy)]
enum XmasLightColor {
    Red,
    Yellow,
    Green,
}

#[derive(Component)]
struct XmasLight(XmasLightColor);

const INTERACTABLE_ID: &str = "light-switch";

const SWITCH_VOLUME: f32 = 0.40;

// Light effect colors.
const ATTIC_LIGHT_COLORS: [Color; 3] = [
    Color::srgb(1.0, 0.6, 0.2),
    Color::srgb(1.0, 0.7, 0.1),
    Color::srgb(1.0, 0.5, 0.3),
];

const XMAS_LIGHT_RED_COLORS: [Color; 3] = [
    Color::srgb(1.0, 0.1, 0.1),
    Color::srgb(0.95, 0.05, 0.05),
    Color::srgb(0.9, 0.0, 0.0),
];

const XMAS_LIGHT_YELLOW_COLORS: [Color; 3] = [
    Color::srgb(1.0, 0.95, 0.2),
    Color::srgb(0.9, 0.9, 0.15),
    Color::srgb(0.95, 0.85, 0.1),
];

const XMAS_LIGHT_GREEN_COLORS: [Color; 3] = [
    Color::srgb(0.1, 1.0, 0.1),
    Color::srgb(0.05, 0.95, 0.05),
    Color::srgb(0.0, 0.9, 0.0),
];

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
fn handle_interaction(mut events: MessageReader<InteractionEvent>, mut query: Query<&mut State, With<Switch>>) {
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
    sprite_assets: Res<SpriteAssets>,
    parent_query: Query<(&Children, &State, &mut Sprite), (With<Switch>, With<InRange>, Changed<State>)>,
    mut light_query: Query<(Entity, &mut PointLight2d, Option<&AtticLight>, Option<&XmasLight>)>,
) {
    let mut rng = rand::rng();

    // Find the child light entity.
    for (children, state, mut sprite) in parent_query {
        for child in children.iter() {
            if let Ok((entity, mut light, attic_light, xmas_light)) = light_query.get_mut(child) {
                match *state {
                    State::On => {
                        sprite.image = sprite_assets.switch_on.clone();

                        commands.spawn((
                            AudioPlayer::new(audio_assets.on.clone()),
                            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(SWITCH_VOLUME)),
                        ));

                        if attic_light.is_some() {
                            let colors = ATTIC_LIGHT_COLORS.to_vec();

                            commands.entity(entity).insert(FlickeringLight {
                                seed: rng.random_range(0.0..1000.0),
                                intensity_amplitude: 0.2,
                                intensity_frequency: 2.0,
                                intensity_min: 0.3,
                                intensity_octaves: 4,
                                color_frequency: 100.0,
                                color_octaves: 5,
                                color_seed_offset: 100.0,
                                color_temperature: 0.5,
                                colors,
                                time_offset: rng.random_range(0.0..100.0),
                            });
                        }

                        if let Some(XmasLight(color)) = xmas_light {
                            match color {
                                XmasLightColor::Red => {
                                    let colors = XMAS_LIGHT_RED_COLORS.to_vec();

                                    commands.entity(entity).insert(FlickeringLight {
                                        seed: rng.random_range(0.0..1000.0),
                                        intensity_amplitude: 0.1,
                                        intensity_frequency: 2.0,
                                        intensity_min: 0.15,
                                        intensity_octaves: 2,
                                        color_frequency: 10.0,
                                        color_octaves: 4,
                                        color_seed_offset: 100.0,
                                        color_temperature: 0.5,
                                        colors,
                                        time_offset: rng.random_range(0.0..100.0),
                                    });
                                }

                                XmasLightColor::Yellow => {
                                    let colors = XMAS_LIGHT_YELLOW_COLORS.to_vec();

                                    commands.entity(entity).insert(FlickeringLight {
                                        seed: rng.random_range(0.0..1000.0),
                                        intensity_amplitude: 0.1,
                                        intensity_frequency: 2.0,
                                        intensity_min: 0.15,
                                        intensity_octaves: 2,
                                        color_frequency: 10.0,
                                        color_octaves: 4,
                                        color_seed_offset: 100.0,
                                        color_temperature: 0.5,
                                        colors,
                                        time_offset: rng.random_range(0.0..100.0),
                                    });
                                }

                                XmasLightColor::Green => {
                                    let colors = XMAS_LIGHT_GREEN_COLORS.to_vec();

                                    commands.entity(entity).insert(FlickeringLight {
                                        seed: rng.random_range(0.0..1000.0),
                                        intensity_amplitude: 0.1,
                                        intensity_frequency: 2.0,
                                        intensity_min: 0.1,
                                        intensity_octaves: 2,
                                        color_frequency: 10.0,
                                        color_octaves: 4,
                                        color_seed_offset: 100.0,
                                        color_temperature: 0.5,
                                        colors,
                                        time_offset: rng.random_range(0.0..100.0),
                                    });
                                }
                            }
                        }
                    }

                    State::Off => {
                        sprite.image = sprite_assets.switch_off.clone();

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
    // Load the sprite sheets.
    let sprites = SpriteAssets {
        switch_on: asset_server.load("house/light_switch_on.png"),
        switch_off: asset_server.load("house/light_switch_off.png"),
        xmas_light_red: asset_server.load("house/xmas_light_red.png"),
        xmas_light_yellow: asset_server.load("house/xmas_light_yellow.png"),
        xmas_light_green: asset_server.load("house/xmas_light_green.png"),
    };
    commands.insert_resource(sprites.clone());

    let audio = AudioAssets {
        on: asset_server.load("house/light_switch_on.ogg"),
        off: asset_server.load("house/light_switch_off.ogg"),
    };
    commands.insert_resource(audio);

    // Parent position is the hidden switch.
    let parent = commands
        .spawn((
            Switch,
            State::Off,
            Sprite {
                image: sprites.switch_off,
                ..default()
            },
            Transform::from_xyz(148.0, -50.0, 5.0),
            Interactable {
                id: INTERACTABLE_ID.to_string(),
                height: 4.0,
                width: 3.0,
                ..default()
            },
        ))
        .id();

    // Spawn light, Local offset from switch (-21, 110, 0) → Global position (128, 60, 5)
    let light_id = commands
        .spawn((
            AtticLight,
            Transform::from_xyz(-20.0, 110.0, 0.0),
            PointLight2d {
                color: ATTIC_LIGHT_COLORS[0],
                intensity: 0.0,
                radius: 160.0,
                cast_shadows: true,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(light_id);

    let x_offset: f32 = -228.0;
    let y: f32 = 55.0;
    for point in 0..40i16 {
        let x = f32::from(point).mul_add(7.0, x_offset);

        match point % 3 {
            0 => {
                let light_id = commands
                    .spawn((
                        XmasLight(XmasLightColor::Yellow),
                        Sprite {
                            image: sprites.xmas_light_yellow.clone(),
                            ..default()
                        },
                        Transform::from_xyz(x, y, 2.0),
                        PointLight2d {
                            color: XMAS_LIGHT_YELLOW_COLORS[0],
                            intensity: 0.0,
                            radius: 15.0,
                            cast_shadows: true,
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(parent).add_child(light_id);
            }

            1 => {
                let light_id = commands
                    .spawn((
                        XmasLight(XmasLightColor::Green),
                        Sprite {
                            image: sprites.xmas_light_green.clone(),
                            ..default()
                        },
                        Transform::from_xyz(x, y, 2.0),
                        PointLight2d {
                            color: XMAS_LIGHT_GREEN_COLORS[0],
                            intensity: 0.0,
                            radius: 15.0,
                            cast_shadows: true,
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(parent).add_child(light_id);
            }

            _ => {
                let light_id = commands
                    .spawn((
                        XmasLight(XmasLightColor::Red),
                        Sprite {
                            image: sprites.xmas_light_red.clone(),
                            ..default()
                        },
                        Transform::from_xyz(x, y, 2.0),
                        PointLight2d {
                            color: XMAS_LIGHT_RED_COLORS[0],
                            intensity: 0.0,
                            radius: 15.0,
                            cast_shadows: true,
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(parent).add_child(light_id);
            }
        }
    }
}
