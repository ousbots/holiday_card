use bevy::prelude::*;
use std::time::Duration;

use crate::{
    attic_light, background, chair, fireplace, flickering_light, house, interaction, santa, snow, snowman, stereo,
    theman, tree,
};

#[derive(Component)]
pub struct AnimationConfig {
    pub first_index: usize,
    pub last_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_index: first,
            last_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / f32::from(fps)), TimerMode::Once)
    }
}

// Add the animation systems.
pub fn add_systems(app: &mut App) {
    attic_light::add_systems(app);
    background::add_systems(app);
    chair::add_systems(app);
    interaction::add_systems(app);
    flickering_light::add_systems(app);
    house::add_systems(app);
    fireplace::add_systems(app);
    santa::add_systems(app);
    snow::add_systems(app);
    snowman::add_systems(app);
    stereo::add_systems(app);
    theman::add_systems(app);
    tree::add_systems(app);
}
