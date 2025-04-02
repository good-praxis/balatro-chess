use bevy::prelude::*;

mod board;
use board::Board;

mod moves;
mod pieces;

mod debug;
use debug::ChessDebugPlugin;

mod zobrist;

pub struct ChessEnginePlugin;
impl Plugin for ChessEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>().add_plugins(ChessDebugPlugin);
    }
}
