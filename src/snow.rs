use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct SnowParticle {
    fall_speed: f32,
    drift_speed: f32,
    drift_phase: f32,
}

#[derive(Component)]
struct Snow;

const PARTICLE_COUNT: usize = 1000;

const SPAWN_Y: f32 = 100.0;
const DESPAWN_Y: f32 = -80.0;
const SPAWN_X_MIN: f32 = -150.0;
const SPAWN_X_MAX: f32 = 150.0;

const FALL_SPEED_MIN: f32 = 10.0;
const FALL_SPEED_MAX: f32 = 30.0;

const DRIFT_SPEED_MIN: f32 = -7.0;
const DRIFT_SPEED_MAX: f32 = 7.0;

const OPACITY_MIN: f32 = 0.4;
const OPACITY_MAX: f32 = 1.0;

// Add the snow systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Startup, init)
        .add_systems(Update, (handle_snow, handle_snow_respawn));
}

// Handle snow particle movement with vertical falling and horizontal wind drift.
fn handle_snow(time: Res<Time>, mut query: Query<(&mut Transform, &SnowParticle), With<Snow>>) {
    for (mut transform, particle) in &mut query {
        let delta = time.delta_secs();

        // Vertical fall with a constant speed per particle.
        transform.translation.y -= particle.fall_speed * delta;

        // Horizontal drift with a sine wave for motion.
        let drift_offset = (time.elapsed_secs() + particle.drift_phase).sin();
        transform.translation.x += particle.drift_speed * drift_offset * delta;
    }
}

// Respawn snow particles that have fallen below the screen.
fn handle_snow_respawn(mut query: Query<(&mut Transform, &mut Sprite, &mut SnowParticle), With<Snow>>) {
    let mut rng = rand::rng();

    for (mut transform, mut sprite, mut particle) in &mut query {
        if transform.translation.y < DESPAWN_Y {
            transform.translation.x = rng.random_range(SPAWN_X_MIN..=SPAWN_X_MAX);
            transform.translation.y = SPAWN_Y;

            particle.fall_speed = rng.random_range(FALL_SPEED_MIN..=FALL_SPEED_MAX);
            particle.drift_speed = rng.random_range(DRIFT_SPEED_MIN..=DRIFT_SPEED_MAX);
            particle.drift_phase = rng.random_range(0.0..=std::f32::consts::TAU);

            let opacity = rng.random_range(OPACITY_MIN..=OPACITY_MAX);
            sprite.color = Color::srgba(1.0, 1.0, 1.0, opacity);
        }
    }
}

// Initialize snow particles distributed across the screen.
fn init(mut commands: Commands) {
    let mut rng = rand::rng();

    for _ in 0..PARTICLE_COUNT {
        let x = rng.random_range(SPAWN_X_MIN..=SPAWN_X_MAX);
        let y = rng.random_range(DESPAWN_Y..=SPAWN_Y);
        let opacity = rng.random_range(OPACITY_MIN..=OPACITY_MAX);

        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, opacity),
                custom_size: Some(Vec2::splat(1.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 1.0)),
            SnowParticle {
                fall_speed: rng.random_range(FALL_SPEED_MIN..=FALL_SPEED_MAX),
                drift_speed: rng.random_range(DRIFT_SPEED_MIN..=DRIFT_SPEED_MAX),
                drift_phase: rng.random_range(0.0..=std::f32::consts::TAU),
            },
            Snow,
        ));
    }
}
