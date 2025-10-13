use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

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
pub struct PixelPerfectColliderEntities(Vec<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
struct ColliderCacheMap(HashMap<AssetId<Image>, Vec<Collider>>);

#[allow(clippy::type_complexity)]
fn add_pixel_perfect_collider(
    sprites: Query<
        (Entity, &Sprite),
        (
            With<PixelPerfectCollider>,
            Without<PixelPerfectColliderEntities>,
        ),
    >,
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

        let entities = colliders
            .into_iter()
            .map(|collider| {
                commands
                    .spawn((
                        Transform::default(),
                        Name::new("PixelPerfectCollider {index}"),
                        ChildOf(entity),
                        //ColliderDisabled,
                        collider,
                    ))
                    .id()
            })
            .collect();

        commands
            .entity(entity)
            .insert((Sensor, PixelPerfectColliderEntities(entities)));
    }
}

fn build_sprite_colliders(image: &Image, regions: &[URect]) -> Vec<Collider> {
    regions
        .iter()
        .map(|&region| build_region_collider(image, region))
        .collect()
}

fn build_region_collider(image: &Image, region: URect) -> Collider {
    let pixels = (region.min.x..region.max.x)
        .flat_map(move |x| (region.min.y..region.max.y).map(move |y| (x, y)))
        .filter_map(|(x, y)| {
            let color = image.get_color_at(x, y).expect("Format to be valid");
            if color.alpha() > 0.0 {
                Some(IVec2::new(x as i32, y as i32))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Collider::voxels(Vec2::splat(1.0), &pixels)
}
