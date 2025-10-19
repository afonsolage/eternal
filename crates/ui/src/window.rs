use bevy::{prelude::*, ui_widgets::observe};

const WINDOW_BACKGROUND_COLOR: Color = Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.9));
const TITLE_BACKGROUND_COLOR: Color = Color::Srgba(Srgba::new(0.3, 0.3, 0.3, 1.0));

#[derive(Component)]
struct WindowRoot;

#[derive(Component)]
struct TitleBar;

#[derive(Component)]
struct WindowBody;

#[derive(Default, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub top: Val,
    pub right: Val,
    pub left: Val,
    pub bottom: Val,
}

pub fn window(
    WindowConfig {
        title,
        top,
        right,
        left,
        bottom,
    }: WindowConfig,
    body: impl Bundle,
) -> impl Bundle {
    (
        BackgroundColor(WINDOW_BACKGROUND_COLOR),
        Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(5.0)),
            top,
            right,
            left,
            bottom,
            ..default()
        },
        WindowRoot,
        children![
            (
                BackgroundColor(TITLE_BACKGROUND_COLOR),
                Name::new("Top"),
                Node {
                    height: px(28.0),
                    ..default()
                },
                children![close_button(), title_bar(title), collapse_button(),],
            ),
            (
                Node {
                    padding: px(10.0).all(),
                    ..default()
                },
                Name::new("Body"),
                WindowBody,
                children![body],
            )
        ],
    )
}

fn close_button() -> impl Bundle {
    (
        Name::new("Left"),
        Node {
            min_width: Val::Px(30.0),
            ..default()
        },
        Text::new("X"),
        TextFont {
            font_size: 25.0,
            ..default()
        },
        TextLayout {
            justify: Justify::Center,
            ..Default::default()
        },
        observe(
            |click: On<Pointer<Click>>, q_hierarchy: Query<&ChildOf>, mut commands: Commands| {
                let root = q_hierarchy.root_ancestor(click.entity);
                commands.entity(root).despawn();
            },
        ),
    )
}

fn collapse_button() -> impl Bundle {
    (
        Name::new("Right"),
        Node {
            min_width: Val::Px(30.0),
            ..default()
        },
        Text::new("-"),
        TextFont {
            font_size: 25.0,
            ..default()
        },
        TextLayout {
            justify: Justify::Center,
            ..Default::default()
        },
        observe(
            |click: On<Pointer<Click>>,
             q_parents: Query<&ChildOf>,
             q_children: Query<&Children>,
             mut q_body: Query<&mut Node, With<WindowBody>>,
             mut commands: Commands,
             mut collapsed: Local<bool>| {
                let root = q_parents.root_ancestor(click.entity);
                if let Some(body) = q_children
                    .iter_descendants(root)
                    .find(|e| q_body.contains(*e))
                    && let Ok(mut body_node) = q_body.get_mut(body)
                {
                    body_node.display = if *collapsed {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }

                if *collapsed {
                    commands.entity(click.entity).insert(Text::new("-"));
                } else {
                    commands.entity(click.entity).insert(Text::new("+"));
                }

                *collapsed = !*collapsed;
            },
        ),
    )
}

fn title_bar(title: impl Into<String>) -> impl Bundle {
    (
        Name::new("Middle"),
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        TitleBar,
        Text::new(title),
        TextFont {
            font_size: 25.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextLayout::new_with_justify(Justify::Center),
        observe(
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
        ),
    )
}
