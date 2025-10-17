use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

use crate::effects::FxIndexChanged;
use eternal_grid::tile;

pub struct PixelPerfectPlugin;

impl Plugin for PixelPerfectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColliderCacheMap>()
            .add_systems(Update, add_pixel_perfect_collider);
    }
}

#[derive(Component)]
pub struct PixelPerfectCollider;

#[derive(Component)]
#[component(immutable)]
struct EntityColliderPairs(Vec<(Entity, Collider)>);

#[derive(Debug, EntityEvent)]
#[entity_event(propagate, auto_propagate)]
pub struct PixelPerfectCollision {
    #[event_target]
    pub source: Entity,
    pub target: Entity,

    #[expect(unused, reason = "This will be used in the future")]
    pub index: usize,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ColliderCacheMap(HashMap<AssetId<Image>, Vec<Collider>>);

fn add_pixel_perfect_collider(
    sprites: Query<(Entity, &Sprite), (With<PixelPerfectCollider>, Without<EntityColliderPairs>)>,
    images: Res<Assets<Image>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut cache: ResMut<ColliderCacheMap>,
    mut commands: Commands,
) {
    for (entity, sprite) in sprites {
        let Some(image) = images.get(sprite.image.id()) else {
            continue;
        };

        let colliders = if let Some(colliders) = cache.get(&sprite.image.id()) {
            colliders.clone()
        } else {
            let colliders = if let Some(atlas) = &sprite.texture_atlas {
                let Some(layout) = layouts.get(atlas.layout.id()) else {
                    error!("Layout not found for sprite pixel perfect");
                    return;
                };
                build_sprite_colliders(image, &layout.textures)
            } else {
                let regions = [URect::new(0, 0, image.width(), image.height())];
                build_sprite_colliders(image, &regions)
            };
            cache.insert(sprite.image.id(), colliders.clone());
            colliders
        };

        let active_index = sprite
            .texture_atlas
            .as_ref()
            .map(|atlas| atlas.index)
            .unwrap_or_default();

        let entities = colliders
            .into_iter()
            .enumerate()
            .map(|(index, collider)| {
                let entity = commands
                    .spawn((
                        Name::new(format!("PixelPerfectCollider {index}")),
                        ChildOf(entity),
                        Sensor,
                        CollisionEventsEnabled,
                    ))
                    .observe(move |add: On<CollisionStart>, mut commands: Commands| {
                        commands.trigger(PixelPerfectCollision {
                            source: add.collider1,
                            target: add.collider2,
                            index,
                        });
                    })
                    .id();

                if index == active_index {
                    commands.entity(entity).insert(collider.clone());
                }

                (entity, collider)
            })
            .collect();

        commands
            .entity(entity)
            .insert(EntityColliderPairs(entities))
            .observe(on_index_changed_update_collider);
    }
}

fn on_index_changed_update_collider(
    index_changed: On<FxIndexChanged>,
    q_entity_collider_pairs: Query<&EntityColliderPairs>,
    mut commands: Commands,
) {
    let Ok(EntityColliderPairs(pairs)) = q_entity_collider_pairs.get(index_changed.entity) else {
        return;
    };

    let (old_entity, _) = pairs[index_changed.old_index];
    commands.entity(old_entity).remove::<Collider>();

    // Collider is cheap to clone, just 2 Arcs and one Vec2
    let (entity, collider) = pairs[index_changed.new_index].clone();
    commands.entity(entity).insert(collider);
}

fn build_sprite_colliders(image: &Image, regions: &[URect]) -> Vec<Collider> {
    regions
        .iter()
        .map(|&region| build_region_collider(image, region))
        .collect()
}

fn build_region_collider(image: &Image, region: URect) -> Collider {
    const TILE_HEIGHT: i32 = tile::SIZE.y as i32 - 1;
    const TILE_HALF_SIZE: IVec2 = IVec2::new(tile::SIZE.x as i32 / 2, tile::SIZE.y as i32 / 2);

    let pixels = (region.min.x..region.max.x)
        .flat_map(move |x| (region.min.y..region.max.y).map(move |y| (x, y)))
        .filter_map(|(x, y)| {
            let color = image.get_color_at(x, y).expect("Format to be valid");
            if color.alpha() > 0.0 {
                // since this may be a region in a texture atlas, convert to "local" coordinates
                let local_x = x as i32 - region.min.x as i32;

                // flip y, since textures works with origin at top-left,
                let local_y = TILE_HEIGHT - (y as i32 - region.min.y as i32);

                // Move to offset from center to the bottom-left
                Some(IVec2::new(local_x, local_y) - TILE_HALF_SIZE)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Collider::voxels(Vec2::splat(1.0), &pixels)
}
