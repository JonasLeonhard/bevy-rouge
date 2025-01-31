use bevy::prelude::*;

use crate::components::Player;

#[derive(Component)]
pub struct FollowedByCamera;

pub(super) fn plugin(app: &mut App) {
    // Spawn the main camera.
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, (movement, follow_player));
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

fn movement(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    player_query: Query<Entity, (With<Player>, Without<Camera>)>,
) {
    let Ok(player_entity) = player_query.get_single() else {
        return;
    };

    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // jump to the player position
        if keyboard_input.just_pressed(KeyCode::Space) {
            commands.entity(player_entity).insert(FollowedByCamera);
            return;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
            commands.entity(player_entity).remove::<FollowedByCamera>();
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
            commands.entity(player_entity).remove::<FollowedByCamera>();
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
            commands.entity(player_entity).remove::<FollowedByCamera>();
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
            commands.entity(player_entity).remove::<FollowedByCamera>();
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

fn follow_player(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<Player>, With<FollowedByCamera>, Without<Camera>)>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let lerp_speed = 5.;
    let target = player_transform.translation;
    let factor = (lerp_speed * time.delta_secs()).min(1.0);
    camera_transform.translation = camera_transform.translation.lerp(target, factor);
}
