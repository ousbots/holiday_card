use crate::animation;

use bevy::{
    audio::{AudioPlugin, SpatialScale},
    prelude::*,
};

const AUDIO_SCALE: f32 = 1. / 500.;

pub fn run_app() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(AudioPlugin {
        default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
        ..default()
    }));
    animation::add_systems(&mut app);

    app.run();
}
