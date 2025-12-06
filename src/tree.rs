use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationConfig,
    interaction::{Highlight, Interactable, InteractionEvent},
};

#[derive(Clone, Component, Copy, PartialEq)]
enum State {
    Off,
    On,
}

#[derive(Clone, Resource)]
struct SpriteAssets {
    on_sprite: Handle<Image>,
    on_layout: Handle<TextureAtlasLayout>,
    off_sprite: Handle<Image>,
}

#[derive(Component)]
struct Tree;

#[derive(Component)]
struct LightTransition {
    current_color: Color,
    target_color: Color,
    current_intensity: f32,
    target_intensity: f32,
    progress: f32,
    transition_duration: f32,
}

const INTERACTABLE_ID: &str = "tree";

const SPRITE_WIDTH: f32 = 14.;
const SPRITE_HEIGHT: f32 = 16.;

const LIGHT_INTENSITY_RANGE: std::ops::RangeInclusive<f32> = 0.2..=0.5;
const LIGHT_RADIUS: f32 = 60.0;

const LIGHT_COLORS: [Color; 3] = [
    Color::srgb(0.2, 0.2, 0.8),
    Color::srgb(0.2, 0.8, 0.2),
    Color::srgb(0.8, 0.2, 0.2),
];

const LIGHT_CHANGE_DELAY: f32 = 0.75;

impl LightTransition {
    // A new randomized light transition.
    fn new() -> Self {
        Self {
            current_color: Self::random_color(),
            target_color: Self::random_color(),
            current_intensity: 0.0,
            target_intensity: Self::random_intensity(),
            progress: 0.0,
            transition_duration: LIGHT_CHANGE_DELAY,
        }
    }

    // Randomize the current light transition.
    fn randomize(&mut self) {
        let mut new_color = Self::random_color();
        while new_color == self.current_color {
            new_color = Self::random_color();
        }
        self.current_color = self.target_color;
        self.target_color = new_color;
        self.current_intensity = self.target_intensity;
        self.target_intensity = Self::random_intensity();
        self.progress = 0.0;
    }

    fn random_color() -> Color {
        LIGHT_COLORS[rand::rng().random_range(0..LIGHT_COLORS.len())]
    }

    fn random_intensity() -> f32 {
        rand::rng().random_range(LIGHT_INTENSITY_RANGE)
    }
}

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
            handle_light_state,
            handle_light_transition,
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

// Apply a pulsing scale effect to highlight the tree is interactable.
fn handle_highlight(
    time: Res<Time>,
    query: Query<(&State, &mut Sprite, &mut Transform, &Highlight, &Interactable), (With<Tree>, With<Highlight>)>,
) {
    for (state, mut sprite, mut transform, highlight, interactable) in query {
        if *state == State::Off && interactable.first {
            let pulse = (((time.elapsed_secs() - highlight.elapsed_offset) * 4.).sin() + 1.).mul_add(0.1, 1.);
            sprite.color = Color::srgba(pulse, pulse, pulse, 1.);
            transform.scale = Vec3::splat(((pulse - 1.) / 4.) + 1.);
        } else {
            sprite.color = Color::WHITE;
            transform.scale = Vec3::splat(1.);
        }
    }
}

// Reset the sprite when the highlight is removed.
fn handle_highlight_reset(
    mut removed: RemovedComponents<Highlight>,
    mut query: Query<(&mut Sprite, &mut Transform), With<Tree>>,
) {
    for entity in removed.read() {
        if let Ok((mut sprite, mut transform)) = query.get_mut(entity) {
            sprite.color = Color::WHITE;
            transform.scale = Vec3::splat(1.);
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

// Disable the highlighting after the first time the tree has been interacted with.
fn handle_interaction_disable_highlight(
    mut query: Query<(&mut State, &mut Interactable), (With<Tree>, Changed<State>)>,
) {
    for (state, mut interactable) in &mut query {
        if *state == State::On {
            interactable.first = false;
        }
    }
}

// Adjust light intensity based on the tree state.
fn handle_light_state(
    mut commands: Commands,
    mut query: Query<(Entity, &State, &mut PointLight2d), (With<Tree>, Changed<State>)>,
) {
    for (entity, state, mut light) in &mut query {
        match *state {
            State::On => {
                commands.entity(entity).insert(LightTransition::new());
            }
            State::Off => {
                light.intensity = 0.0;
                commands.entity(entity).remove::<LightTransition>();
            }
        }
    }
}

// Smoothly transition the light color and intensity over time when the tree is on.
fn handle_light_transition(time: Res<Time>, mut query: Query<(&mut PointLight2d, &mut LightTransition), With<Tree>>) {
    for (mut light, mut transition) in &mut query {
        transition.progress += time.delta().as_secs_f32() / transition.transition_duration;

        // Pick a new target when the current transition is complete.
        if transition.progress >= 1.0 {
            transition.randomize();
            continue;
        }

        // Apply sine-wave easing and interpolate.
        let current_rgb = transition.current_color.to_srgba();
        let target_rgb = transition.target_color.to_srgba();
        let eased_t = (1.0 - (std::f32::consts::PI * transition.progress).cos()) / 2.0;

        light.color = Color::srgb(
            (target_rgb.red - current_rgb.red).mul_add(eased_t, current_rgb.red),
            (target_rgb.green - current_rgb.green).mul_add(eased_t, current_rgb.green),
            (target_rgb.blue - current_rgb.blue).mul_add(eased_t, current_rgb.blue),
        );

        light.intensity =
            (transition.target_intensity - transition.current_intensity).mul_add(eased_t, transition.current_intensity);
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
        Transform::from_translation(Vec3::new(-90.0, -38.0, 5.0)),
        Tree,
        AnimationConfig::new(0, 4, 2),
        State::Off,
        Interactable {
            id: INTERACTABLE_ID.to_string(),
            height: SPRITE_HEIGHT,
            width: SPRITE_WIDTH,
            first: true,
        },
        PointLight2d {
            color: LIGHT_COLORS[0],
            intensity: 0.0,
            radius: LIGHT_RADIUS,
            ..default()
        },
    ));
}
