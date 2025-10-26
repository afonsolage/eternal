use bevy::{
    feathers::controls::checkbox,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, observe},
};
use eternal_ui::window::{WindowConfig, window};

use crate::EditorState;

pub struct MapUiPlugin;

impl Plugin for MapUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(EditorState::Map), setup)
            .add_systems(OnExit(EditorState::Map), cleanup);
    }
}

#[derive(Resource, Reflect)]
pub struct MapOptions {
    pub terrain: bool,
    pub flora: bool,
}

#[derive(Component)]
struct MapUi;

fn setup(mut commands: Commands) {
    commands.insert_resource(MapOptions {
        terrain: true,
        flora: true,
    });

    commands.spawn((
        window(
            WindowConfig {
                title: "Options".to_string(),
                top: px(1.0),
                right: px(1.0),
                ..default()
            },
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    (
                        checkbox((Checked,), Spawn(Text::new("Terrain"))),
                        observe(
                            |change: On<ValueChange<bool>>,
                             mut opts: ResMut<MapOptions>,
                             mut commands: Commands| {
                                opts.terrain = change.value;
                                if opts.terrain {
                                    commands.entity(change.event_target()).insert(Checked);
                                } else {
                                    commands.entity(change.event_target()).remove::<Checked>();
                                }
                            }
                        )
                    ),
                    (
                        checkbox((Checked,), Spawn(Text::new("Flora"))),
                        observe(
                            |change: On<ValueChange<bool>>,
                             mut opts: ResMut<MapOptions>,
                             mut commands: Commands| {
                                opts.flora = change.value;
                                if opts.flora {
                                    commands.entity(change.event_target()).insert(Checked);
                                } else {
                                    commands.entity(change.event_target()).remove::<Checked>();
                                }
                            }
                        )
                    ),
                ],
            ),
        ),
        MapUi,
    ));
}

fn cleanup(ui_root: Single<Entity, With<MapUi>>, mut commands: Commands) {
    commands.entity(ui_root.into_inner()).despawn();
}
