use bevy::{
    app::Plugin,
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
};

pub mod controls;
pub mod window;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()));
    }
}
