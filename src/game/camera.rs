use bevy::prelude::*;

use crate::components::Player;

pub(super) fn plugin(app: &mut App) {
    // Spawn the main camera.
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, movement);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}

pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // jump to the player position
        if keyboard_input.just_pressed(KeyCode::Space) {
            if let Ok(player_transform) = player_query.get_single() {
                transform.translation = player_transform.translation;
                return;
            }
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::PageUp) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::PageDown) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        transform.translation.x += time.delta_secs() * direction.x * 500.;
        transform.translation.y += time.delta_secs() * direction.y * 500.;
    }
}
