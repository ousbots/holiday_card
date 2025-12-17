use bevy::{
    audio::{AudioPlugin, SpatialScale},
    prelude::*,
};
use bevy_light_2d::prelude::*;

use crate::{animation, camera, input};

const AUDIO_SCALE: f32 = 1. / 200.;

pub fn run_app() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()).set(AudioPlugin {
            default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
            ..default()
        }),
        Light2dPlugin,
    ));
    camera::add_systems(&mut app);
    input::add_systems(&mut app);
    animation::add_systems(&mut app);

    app.run();
}
