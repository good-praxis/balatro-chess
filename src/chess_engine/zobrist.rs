use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use strum::IntoEnumIterator;

use super::{
    game::Game,
    moves::Ply,
    pieces::{Piece, PieceType},
};

#[derive(Debug, Hash, PartialEq, Eq)]
enum ZobristKey {
    Piece(PieceType, usize),
}

#[derive(Debug)]
pub struct Zobrist {
    table: HashMap<ZobristKey, u32>,
}

impl Zobrist {
    pub fn new(board_size: usize) -> Self {
        // 24337 = chess on a phone keyboard
        let mut rng = ChaCha8Rng::seed_from_u64(24337);
        let mut table = HashMap::new();

        for piece_type in PieceType::iter() {
            for i in 0..board_size {
                table.insert(ZobristKey::Piece(piece_type, i), rng.random());
            }
        }

        Self { table }
    }

    pub fn gen_initial_hash(&self, board: &Vec<Option<Piece>>) -> u32 {
        let mut hash = 0;
        for (i, tile) in board.iter().enumerate() {
            if let Some(piece) = tile {
                hash ^= self.table[&ZobristKey::Piece(piece.piece_type, i)];
            }
        }

        hash
    }

    /// Function works in both directions due to the xoring
    pub fn update_hash(&self, board: &Game, mut hash: u32, ply: Ply) -> u32 {
        // remove previous position for moving piece
        hash ^=
            self.table[&ZobristKey::Piece(ply.by.piece_type, board.pos_to_idx(ply.move_to.from))];
        // add new position for moving piece
        hash ^= self.table[&ZobristKey::Piece(ply.by.piece_type, board.pos_to_idx(ply.move_to.to))];
        // remove captured piece position
        if let Some(captured) = ply.capturing {
            hash ^=
                self.table[&ZobristKey::Piece(captured.piece_type, board.pos_to_idx(captured.pos))];
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        moves::{MoveTo, Pos},
        pieces::PieceColor,
    };

    use super::*;

    #[test]
    fn hash_on_default() {
        let board = Game::default();

        assert_ne!(board.zobrist_hash, 0);
    }

    #[test]
    fn hash_same_seed() {
        let board = Game::default();
        let other_board = Game::default();

        assert_eq!(board.zobrist_hash, other_board.zobrist_hash);
    }

    #[test]
    fn hash_updates() {
        let mut board = Game::default();
        let ply = Ply {
            move_to: MoveTo {
                from: Pos::new(7, 1),
                to: Pos::new(5, 0),
            },
            by: Piece {
                piece_type: PieceType::Knight,
                color: PieceColor::White,
                pos: Pos::new(7, 1),
                has_moved: false,
            },
            capturing: None,
            en_passant_flag: false,
        };
        let hash_before = board.zobrist_hash;
        board.apply_ply(ply);
        let hash_after = board.zobrist_hash;

        assert_ne!(hash_before, hash_after);
    }

    #[test]
    fn hash_rewinds() {
        let mut board = Game::default();
        let ply = Ply {
            move_to: MoveTo {
                from: Pos::new(7, 1),
                to: Pos::new(5, 0),
            },
            by: Piece {
                piece_type: PieceType::Knight,
                color: PieceColor::White,
                pos: Pos::new(7, 1),
                has_moved: false,
            },
            capturing: None,
            en_passant_flag: false,
        };
        let hash_before = board.zobrist_hash;
        board.apply_ply(ply);
        board.rewind_last_move();
        let hash_after = board.zobrist_hash;

        assert_eq!(hash_before, hash_after);
    }
}
