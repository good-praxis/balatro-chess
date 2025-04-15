use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::{
    bitboard::{Ply, Weights},
    game::Game,
};

#[derive(Resource, Debug, Clone, Copy, Default)]
struct LastPly(Option<Ply>);

#[derive(Resource, Debug, Clone, Default, Deref)]
struct NextBoard(Option<(String, String)>);

pub struct ChessDebugPlugin;
impl Plugin for ChessDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug)
            .add_systems(Update, (find_next_ply, print_new_board))
            .init_resource::<DebugFlags>()
            .init_resource::<LastPly>()
            .init_resource::<NextBoard>();
    }
}

#[derive(Component, Debug)]
struct DebugTextBoard;
#[derive(Component, Debug)]
struct DebugTextInfo;

#[derive(Resource, Debug, Clone)]
struct DebugFlags {
    running: bool,
    waiting_to_print: bool,
}
impl Default for DebugFlags {
    fn default() -> Self {
        Self {
            running: true,
            waiting_to_print: true,
        }
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

fn print_new_board(
    mut board_text_query: Query<&mut Text, (With<DebugTextBoard>, Without<DebugTextInfo>)>,
    mut info_text_query: Query<&mut Text, (With<DebugTextInfo>, Without<DebugTextBoard>)>,
    mut next_board: ResMut<NextBoard>,
    mut debug_flags: ResMut<DebugFlags>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        debug_flags.waiting_to_print = true;
    }

    if debug_flags.waiting_to_print {
        if let NextBoard(Some((board, info))) = next_board.clone() {
            *next_board = NextBoard(None);
            board_text_query.single_mut().0 = board;
            info_text_query.single_mut().0 = info;
            debug_flags.waiting_to_print = false;
        }
    }
}

fn find_next_ply(
    mut game: ResMut<Game>,
    mut last_ply: ResMut<LastPly>,
    mut debug_flags: ResMut<DebugFlags>,
    mut next_board: ResMut<NextBoard>,
) {
    if debug_flags.running && next_board.is_none() {
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

            *next_board = NextBoard(Some((
                game.boards.to_string(),
                format!(
                    "{}\nTime:\n{}\n\n Nodes visited:\n{}",
                    ply.to_string(),
                    work_done.as_millis(),
                    result.2
                ),
            )));

        ////////////////////////////////////////////////////////////////////
        } else {
            let board = game.to_string();
            let info = format!("\n{:?} lost!", game.next_move_by);
            *next_board = NextBoard(Some((board, info)));
            debug_flags.running = false;
        }
    }
}
