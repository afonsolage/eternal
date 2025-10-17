use bevy::{prelude::*, ui_widgets::observe};

use crate::effects::{
    FxAnimation,
    pixel_perfect::{PixelPerfectCollider, PixelPerfectCollision},
};

pub struct SwipePlugin;

impl Plugin for SwipePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_impact_add);
    }
}

#[derive(Component)]
pub struct FxSwipe;

#[derive(EntityEvent)]
pub struct FxSwipeHit {
    #[event_target]
    pub entity: Entity,
    pub source: Entity,
    pub target: Entity,
}

fn on_impact_add(
    add: On<Add, FxSwipe>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sheets/swipe.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        7,
        1,
        None,
        None,
    ));

    commands.entity(add.entity).insert((
        Sprite::from_atlas_image(texture, layout.into()),
        FxAnimation::once(20.0, 0, 6),
        PixelPerfectCollider,
        observe(on_impact_hit),
    ));
}

fn on_impact_hit(collision: On<PixelPerfectCollision>, mut commands: Commands) {
    commands.trigger(FxSwipeHit {
        entity: collision.source,
        source: collision.original_event_target(),
        target: collision.target,
    });
}
