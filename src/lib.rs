mod components;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod resources;
mod screens;
mod states;

use bevy::{
    audio::{AudioPlugin, Volume},
    prelude::*,
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "M Rouge".to_string(),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.3),
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // make nearest sampling default for pixel art
        );

        app.insert_resource(ClearColor(Color::srgb(0., 0., 0.)));

        // Add other plugins.
        app.add_plugins((game::plugin, screens::plugin));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}
