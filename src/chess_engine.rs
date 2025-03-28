use std::time::Duration;

use bevy::prelude::*;

mod board;
use board::Board;

mod moves;
mod pieces;

pub struct ChessEnginePlugin;
impl Plugin for ChessEnginePlugin {
    fn build(&self, app: &mut App) {
        let mut board = Board::default();

        loop {
            println!("{}", board);
            if let Some(next_move) = board.search_next_move(3).1 {
                board.apply_move(next_move);
                std::thread::sleep(Duration::from_secs_f32(1.0));
            } else {
                break;
            }
        }
        app.init_resource::<Board>();
    }
}
