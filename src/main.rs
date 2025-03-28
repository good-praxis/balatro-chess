use bevy::prelude::*;

mod chess_engine;
use chess_engine::*;

fn main() {
    let app_window = Some(Window {
        title: "Chess!".to_string(),
        resolution: (800., 800.).into(),

        resizable: false,
        ..Default::default()
    });

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: app_window,
                ..Default::default()
            }),
            MeshPickingPlugin,
        ))
        .add_plugins(ChessEnginePlugin)
        .run();
}
