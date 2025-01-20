use std::time::Duration;

use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

// TODO: implement actions
#[derive(Component)]
pub struct TurnTaker {
    pub actions_per_turn: u32,
    pub actions_remaining: u32,
}

impl Default for TurnTaker {
    fn default() -> Self {
        Self {
            actions_per_turn: 1,
            actions_remaining: 1,
        }
    }
}

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

#[derive(Component)]
pub struct HighlightBorder;

/// Tiles you cannot move over
#[derive(Component)]
pub struct Obstacle;
