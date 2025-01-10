//! Development tools for the game. This plugin is only enabled in dev builds.
use bevy::{
    dev_tools::{
        states::log_transitions,
        ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::states::{Screen, TurnState};

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<TurnState>);

    // bevy_inspector_egui
    app.add_plugins(WorldInspectorPlugin::new());

    // Toggle the debug overlay for UI.
    app.add_plugins(DebugUiPlugin);
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::F10;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
