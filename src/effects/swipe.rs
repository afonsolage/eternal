use bevy::prelude::*;

use crate::effects::FxAnimation;

pub struct SwipePlugin;

impl Plugin for SwipePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_impact_add);
    }
}

#[derive(Component)]
pub struct FxSwipe;

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
    ));
}
