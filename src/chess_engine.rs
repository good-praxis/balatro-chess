use bevy::prelude::*;

mod game;
pub use game::Game;

mod moves;
mod pieces;

mod debug;
use debug::ChessDebugPlugin;

pub mod bitboard;
mod zobrist;

pub struct ChessEnginePlugin;
impl Plugin for ChessEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>().add_plugins(ChessDebugPlugin);
    }
}
