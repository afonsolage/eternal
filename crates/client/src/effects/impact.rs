use bevy::prelude::*;

use crate::effects::FxAnimation;

pub struct ImpactPlugin;

impl Plugin for ImpactPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_impact_add);
    }
}

#[derive(Component)]
pub struct FxImpact;

fn on_impact_add(
    add: On<Add, FxImpact>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sheets/impact.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        4,
        1,
        None,
        None,
    ));

    commands.entity(add.entity).insert((
        Sprite::from_atlas_image(texture, layout.into()),
        FxAnimation::ping_pong(20.0, 0, 3),
    ));
}
