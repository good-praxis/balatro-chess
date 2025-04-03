use bevy::prelude::*;

mod game;
use game::Game;

mod moves;
mod pieces;

mod debug;
use debug::ChessDebugPlugin;

mod bitboard;
mod zobrist;

pub struct ChessEnginePlugin;
impl Plugin for ChessEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>().add_plugins(ChessDebugPlugin);
    }
}
