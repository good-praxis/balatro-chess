use std::time::Duration;

use bevy::prelude::*;

use super::board::Board;

pub struct ChessDebugPlugin;
impl Plugin for ChessDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug)
            .add_systems(Update, print_and_play)
            .init_resource::<DebugFlags>();
    }
}

#[derive(Component, Debug)]
struct DebugText;

#[derive(Resource, Debug, Clone)]
struct DebugFlags {
    running: bool,
}
impl Default for DebugFlags {
    fn default() -> Self {
        Self { running: true }
    }
}

fn setup_debug(mut commands: Commands, assets: Res<AssetServer>, board: Res<Board>) {
    let font = assets.load("fonts/FSEX300.ttf");
    let text_color = TextColor(Color::WHITE);
    let text_font = TextFont {
        font: font.clone(),
        font_size: 60.0,
        ..default()
    };
    let text_alignment = TextLayout::new_with_justify(JustifyText::Center);
    let text = Text::new(board.to_string());

    commands.spawn(Camera2d);
    commands.spawn((text, text_color, text_font, text_alignment, DebugText));
}

fn print_and_play(
    mut query: Query<&mut Text, With<DebugText>>,
    mut board: ResMut<Board>,
    mut debug_flags: ResMut<DebugFlags>,
) {
    if debug_flags.running {
        std::thread::sleep(Duration::from_secs_f32(0.5));
        if let Some(ply) = board.search_next_move(1).1 {
            board.apply_ply(ply);
            query.single_mut().0 = board.to_string();
        } else {
            let mut string = board.to_string();
            string.push_str(&format!("\n{:?} lost!", board.next_move_by));
            query.single_mut().0 = string;
            debug_flags.running = false;
        }
    }
}
