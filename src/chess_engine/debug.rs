use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::{
    bitboard::{Ply, Weights},
    game::Game,
};

#[derive(Resource, Debug, Clone, Copy, Default)]
struct LastPly(Option<Ply>);

pub struct ChessDebugPlugin;
impl Plugin for ChessDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug)
            .add_systems(Update, print_and_play)
            .init_resource::<DebugFlags>()
            .init_resource::<LastPly>();
    }
}

#[derive(Component, Debug)]
struct DebugTextBoard;
#[derive(Component, Debug)]
struct DebugTextInfo;

#[derive(Resource, Debug, Clone)]
struct DebugFlags {
    running: bool,
}
impl Default for DebugFlags {
    fn default() -> Self {
        Self { running: true }
    }
}

fn setup_debug(mut commands: Commands, assets: Res<AssetServer>, board: Res<Game>) {
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
    commands.spawn((text, text_color, text_font, text_alignment, DebugTextBoard));

    let text_color = TextColor(Color::WHITE);
    let text_font = TextFont {
        font: font.clone(),
        font_size: 20.0,
        ..default()
    };
    let text_alignment = TextLayout::new_with_justify(JustifyText::Right);
    let text = Text::new(board.to_string());

    commands.spawn((
        text,
        text_color,
        text_font,
        text_alignment,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        DebugTextInfo,
    ));
}

fn print_and_play(
    mut board_text_query: Query<&mut Text, (With<DebugTextBoard>, Without<DebugTextInfo>)>,
    mut info_text_query: Query<&mut Text, (With<DebugTextInfo>, Without<DebugTextBoard>)>,
    mut game: ResMut<Game>,
    mut last_ply: ResMut<LastPly>,
    mut debug_flags: ResMut<DebugFlags>,
) {
    if debug_flags.running {
        std::thread::sleep(Duration::from_secs_f32(0.5));

        let start = Instant::now();
        ////////////////////////////////////////////////////////////////////
        // Mailbox impl
        // if let Some(ply) = game.search_next_move(3).1 {
        //     game.apply_ply(ply);
        //     let work_done = Instant::now().duration_since(start);
        //     query.single_mut().0 = game.to_string();
        //     query
        //         .single_mut()
        //         .0
        //         .push_str(&format!("\nTime: {}", work_done.as_millis()));

        // Bitboard impl
        let weights: Weights = Weights {
            king: 4000,
            queen: 180,
            rook: 100,
            bishop: 60,
            knight: 60,
            pawn: 20,
            isolated_pawn: -5,
            movement: 1,
        };
        let result = game.boards.search_next_ply(last_ply.0, 3, weights);
        if let Some(ply) = result.1 {
            game.boards.make_ply(&ply);
            last_ply.0 = Some(ply);
            let work_done = Instant::now().duration_since(start);

            board_text_query.single_mut().0 = game.boards.to_string();
            info_text_query.single_mut().0 = format!(
                "\nTime:\n{}\n\n Nodes visited:\n{}",
                work_done.as_millis(),
                result.2
            );
        ////////////////////////////////////////////////////////////////////
        } else {
            let mut string = game.to_string();
            string.push_str(&format!("\n{:?} lost!", game.next_move_by));
            board_text_query.single_mut().0 = string;
            debug_flags.running = false;
        }
    }
}
