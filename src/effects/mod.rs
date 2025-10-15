#![allow(unused)]
use std::ops::Deref;

use bevy::prelude::*;

use crate::effects::{impact::ImpactPlugin, pixel_perfect::PixelPerfectPlugin, swipe::SwipePlugin};

mod impact;
pub use impact::FxImpact;

mod swipe;
pub use swipe::FxSwipe;

mod pixel_perfect;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FxFpsMultiplier(1.0))
            .add_plugins((PixelPerfectPlugin, ImpactPlugin, SwipePlugin))
            .add_systems(Update, advance_frame);
    }
}

#[derive(EntityEvent)]
pub struct FxIndexChanged {
    pub entity: Entity,
    pub old_index: usize,
    pub new_index: usize,
}

#[derive(Resource, Reflect, Deref)]
pub struct FxFpsMultiplier(pub f32);

#[derive(Default)]
enum LoopType {
    #[default]
    None,
    PingPong,
    Cicle(usize),
}

#[derive(Component)]
struct FxAnimation {
    fps: f32,
    first: usize,
    last: usize,
    elapsed: f32,
    loop_type: LoopType,
}

impl FxAnimation {
    fn once(fps: f32, first: usize, last: usize) -> Self {
        Self {
            fps,
            first,
            last,
            elapsed: first as f32,
            loop_type: LoopType::None,
        }
    }

    fn ping_pong(fps: f32, first: usize, last: usize) -> Self {
        Self {
            fps,
            first,
            last,
            elapsed: first as f32,
            loop_type: LoopType::PingPong,
        }
    }

    fn cycle(fps: f32, first: usize, last: usize, count: usize) -> Self {
        Self {
            fps,
            first,
            last,
            elapsed: first as f32,
            loop_type: LoopType::Cicle(count),
        }
    }
}

fn advance_frame(
    mut q_sprites: Query<(Entity, &mut Sprite, &mut FxAnimation)>,
    mut commands: Commands,
    fps_multi: Res<FxFpsMultiplier>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut anim) in &mut q_sprites {
        // Borrow as shared to avoid unecessary change detection activation
        let Some(atlas) = &sprite.texture_atlas else {
            continue;
        };

        anim.elapsed += time.delta_secs() * anim.fps * (**fps_multi);
        let current = anim.elapsed as usize;

        let next_index = match anim.loop_type {
            LoopType::None => {
                if current > anim.last {
                    commands.entity(entity).despawn();
                    continue;
                } else {
                    current
                }
            }
            LoopType::PingPong => {
                if current > anim.last * 2 {
                    commands.entity(entity).despawn();
                    continue;
                } else if current > anim.last {
                    anim.last * 2 - current
                } else {
                    current
                }
            }
            LoopType::Cicle(cicles) => {
                let current_cicle = current / anim.last;
                if current_cicle > cicles {
                    commands.entity(entity).despawn();
                    continue;
                }
                current % anim.last
            }
        };

        if atlas.index != next_index {
            // Borrow as exclusive triggering change detection;
            let atlas = sprite
                .texture_atlas
                .as_mut()
                .expect("At this point, atlas exists");

            let evt = FxIndexChanged {
                entity,
                old_index: atlas.index,
                new_index: next_index,
            };

            atlas.index = next_index;
            commands.trigger(evt);
        }
    }
}
