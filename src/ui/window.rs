use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

#[derive(Component)]
struct WindowRoot;

#[derive(Component)]
struct CloseBtn;

#[derive(Component)]
struct TitleBar;

pub fn spawn_window<'a>(
    commands: &'a mut Commands,
    asset_server: &'a Res<AssetServer>,
    title: impl Into<String>,
    override_root: impl Bundle,
) -> EntityCommands<'a> {
    let image = asset_server.load("ui/9_slice/window.png");

    let slicer = TextureSlicer {
        border: BorderRect::all(32.0),
        max_corner_scale: 3.0,
        ..Default::default()
    };

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let mut entity_commands = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(1000.0),
            height: Val::Px(500.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        ImageNode {
            image,
            image_mode: NodeImageMode::Sliced(slicer),
            ..Default::default()
        },
        WindowRoot,
        override_root,
    ));

    let parent = entity_commands
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Top"),
                    Node {
                        width: Val::Percent(100.0),
                        height: px(30.0),
                        min_height: px(30.0),
                        margin: UiRect::top(Val::Px(5.0)),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    spawn_close_btn(parent);
                    spawn_title_bar(parent, font, title);

                    // No use, right now, for the right portion  of the window
                    parent.spawn((
                        Name::new("Right"),
                        Node {
                            width: Val::Px(40.0),
                            min_width: Val::Px(40.0),
                            ..Default::default()
                        },
                    ));
                });
        })
        .id();

    commands.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            padding: px(10.0).all(),
            ..Default::default()
        },
        Name::new("Body"),
        ChildOf(parent),
    ))
}

fn spawn_close_btn(parent: &mut RelatedSpawnerCommands<ChildOf>) {
    parent
        .spawn((
            Name::new("Left"),
            Node {
                width: Val::Px(40.0),
                min_width: Val::Px(40.0),
                ..Default::default()
            },
        ))
        .observe(
            |click: On<Pointer<Click>>, q_hierarchy: Query<&ChildOf>, mut commands: Commands| {
                let root = q_hierarchy.root_ancestor(click.entity);
                commands.entity(root).despawn();
            },
        );
}

fn spawn_title_bar(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: Handle<Font>,
    title: impl Into<String>,
) {
    parent
        .spawn((
            Name::new("Middle"),
            Node {
                width: Val::Percent(100.0),
                ..Default::default()
            },
            TitleBar,
            Text::new(title),
            TextFont {
                font,
                font_size: 25.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextLayout::new_with_justify(Justify::Center),
        ))
        .observe(
            |mut drag: On<Pointer<Drag>>,
             q_hierarchy: Query<&ChildOf>,
             mut q_root: Query<&mut UiTransform, With<WindowRoot>>| {
                drag.propagate(false);

                let root = q_hierarchy.root_ancestor(drag.entity);
                let Ok(mut root_transform) = q_root.get_mut(root) else {
                    error!("Failed to get window root. Node not found.");
                    return;
                };

                let Val2 {
                    x: Val::Px(x),
                    y: Val::Px(y),
                } = root_transform.translation
                else {
                    error!(
                        "Invalid UiTransform root: {:?}.",
                        root_transform.translation
                    );
                    return;
                };

                root_transform.translation = Val2::px(x + drag.delta.x, y + drag.delta.y);
            },
        );
}
