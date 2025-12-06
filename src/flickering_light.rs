use bevy::prelude::*;
use bevy_light_2d::prelude::*;

use crate::noise;

#[derive(Component)]
pub struct FlickeringLight {
    pub seed: f32,
    pub intensity_amplitude: f32,
    pub intensity_frequency: f32,
    pub intensity_min: f32,
    pub intensity_octaves: u32,
    pub color_frequency: f32,
    pub color_octaves: u32,
    pub color_seed_offset: f32,
    pub color_temperature: f32,
    pub colors: Vec<Color>,
    pub time_offset: f32,
}

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    app.add_systems(Update, handle_light_flicker);
}

// Blend the colors using weights.
fn blend_colors(colors: &Vec<Color>, weights: &Vec<f32>) -> Color {
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;

    for (color, &weight) in colors.iter().zip(weights.iter()) {
        let srgba = color.to_srgba();
        r += srgba.red * weight;
        g += srgba.green * weight;
        b += srgba.blue * weight;
    }

    Color::srgb(r, g, b)
}

// Apply noise-based flicker to the light color and intensity.
fn handle_light_flicker(time: Res<Time>, mut query: Query<(&mut PointLight2d, &FlickeringLight)>) {
    for (mut light, params) in &mut query {
        let time = time.elapsed_secs() + params.time_offset;

        // Intensity randomization.
        let intensity_noise = noise::generate(time * params.intensity_frequency, params.seed, params.intensity_octaves);
        light.intensity = intensity_noise.mul_add(params.intensity_amplitude, params.intensity_min);

        // Color randomization.
        light.color = blend_colors(
            &params.colors,
            &weights(
                time,
                params.seed,
                params.color_frequency,
                params.colors.len(),
                params.color_octaves,
                params.color_seed_offset,
                params.color_temperature,
            ),
        );
    }
}

// Apply softmax normalization with a temperature parameter.
fn softmax(logits: &[f32], temperature: f32) -> Vec<f32> {
    let scaled_logits: Vec<f32> = logits.iter().map(|x| x / temperature).collect();
    let max_logit = scaled_logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let exp_values: Vec<f32> = scaled_logits.iter().map(|x| (x - max_logit).exp()).collect();
    let sum: f32 = exp_values.iter().sum();

    exp_values.iter().map(|x| x / sum).collect()
}

// Generate weights using softmax normalization of noise-generated logits.
fn weights(
    time: f32,
    seed: f32,
    frequency: f32,
    number: usize,
    octaves: u32,
    seed_offset: f32,
    temperature: f32,
) -> Vec<f32> {
    let mut logits = Vec::with_capacity(number);

    for i in 0..number {
        let color_seed = (i as f32).mul_add(seed_offset, seed);
        let noise_value = noise::generate(time * frequency, color_seed, octaves);
        logits.push(noise_value);
    }

    softmax(&logits, temperature)
}
