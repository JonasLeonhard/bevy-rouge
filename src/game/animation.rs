use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
    pub should_loop: bool,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8, should_loop: bool) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps, should_loop),
            should_loop,
        }
    }

    pub fn timer_from_fps(fps: u8, should_loop: bool) -> Timer {
        let duration = Duration::from_secs_f32(1.0 / (fps as f32));
        let mode = if should_loop {
            TimerMode::Repeating
        } else {
            TimerMode::Once
        };
        Timer::new(duration, mode)
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, execute_animations);
}

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        // we track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        // this only happens for should_loop == false AnimationConfig's
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == config.last_sprite_index {
                    // ...and it IS the last frame, then we move back to the first frame and stop.
                    atlas.index = config.first_sprite_index;
                } else {
                    // ...and it is NOT the last frame, then we move to the next frame...
                    atlas.index += 1;
                    // ...and reset the frame timer to start counting all over again
                    config.frame_timer =
                        AnimationConfig::timer_from_fps(config.fps, config.should_loop);
                }
            }
        }
    }
}
