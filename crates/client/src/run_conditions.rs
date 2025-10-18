use std::time::Duration;

use bevy::prelude::*;

pub fn timeout(duration: Duration) -> impl FnMut(Local<f32>, Res<Time>) -> bool {
    move |mut timer: Local<f32>, time: Res<Time>| {
        *timer += time.delta_secs();
        if *timer >= duration.as_secs_f32() {
            *timer = 0.0;
            true
        } else {
            false
        }
    }
}

pub fn component_changed<T: Component>(q: Query<(), Changed<T>>) -> bool {
    !q.is_empty()
}
