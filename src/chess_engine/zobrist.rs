use super::{
    bitboard::BitIndex,
    game::Game,
    pieces::{LegacyPiece, PIECE_COLOR_COUNT, PIECE_TYPE_COUNT, Piece},
};
use bevy::prelude::Deref;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::ops::BitXorAssign;

pub const PIECE_POSITIONS_COUNT: usize = PIECE_TYPE_COUNT * PIECE_COLOR_COUNT * 256;
pub const CHANGE_PLAYER_INDEX: usize = PIECE_POSITIONS_COUNT;
pub const ZOBRIST_TABLE_LENGTH: usize = PIECE_POSITIONS_COUNT + 1;

#[derive(Debug, Hash, PartialEq, Eq)]
enum ZobristKey {
    Piece(Piece, u32),
    ChangePlayer,
}
impl ZobristKey {
    #[inline]
    fn to_index(&self) -> usize {
        match self {
            Self::Piece(piece, position) => {
                (512 * piece.0 as usize) + (256 * piece.1 as usize) + *position as usize
            }
            Self::ChangePlayer => CHANGE_PLAYER_INDEX,
        }
    }
}

#[derive(Debug, Deref, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZobristHash(u32);

impl From<u32> for ZobristHash {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl BitXorAssign for ZobristHash {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 = **self ^ *rhs;
    }
}

#[derive(Debug)]
pub struct Zobrist {
    pub table: [ZobristHash; ZOBRIST_TABLE_LENGTH],
}
impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

impl Zobrist {
    pub fn new() -> Self {
        // 24337 = chess on a phone keyboard
        let mut rng = ChaCha8Rng::seed_from_u64(24337);
        let mut table = [ZobristHash(0); ZOBRIST_TABLE_LENGTH];

        for i in 0..ZOBRIST_TABLE_LENGTH {
            table[i] = rng.random::<u32>().into();
        }

        Self { table }
    }

    pub fn gen_initial_hash_mailbox(&self, board: &Vec<Option<LegacyPiece>>) -> ZobristHash {
        let mut hash = 0.into();
        for (i, tile) in board.iter().enumerate() {
            if let Some(piece) = tile {
                hash ^= self.table
                    [ZobristKey::Piece(Piece(piece.piece_type, piece.color), i as u32).to_index()];
            }
        }

        hash
    }

    pub fn gen_initial_hash_bitboard(
        &self,
        pieces_iter: impl Iterator<Item = (Piece, BitIndex)>,
    ) -> ZobristHash {
        let mut hash = 0.into();
        for (piece, bitindex) in pieces_iter {
            hash ^= self.table[ZobristKey::Piece(piece, *bitindex).to_index()];
        }

        hash
    }

    /// Function works in both directions due to the xoring
    pub fn update_hash_bitboard(
        &self,
        mut hash: ZobristHash,
        ply: &super::bitboard::Ply,
    ) -> ZobristHash {
        // remove previous position for moving piece
        hash ^= self.table[ZobristKey::Piece(ply.moving_piece, *ply.from).to_index()];
        // add new position for moving piece
        hash ^= self.table[ZobristKey::Piece(ply.moving_piece, *ply.to).to_index()];
        // remove captured piece position
        if let Some(captured) = ply.capturing {
            hash ^= self.table[ZobristKey::Piece(captured.0, *captured.1).to_index()];
        }
        // Change player
        hash ^= self.table[ZobristKey::ChangePlayer.to_index()];

        hash
    }

    /// Function works in both directions due to the xoring
    pub fn update_hash_mailbox(
        &self,
        board: &Game,
        mut hash: ZobristHash,
        ply: super::moves::LegacyPly,
    ) -> ZobristHash {
        // remove previous position for moving piece
        hash ^= self.table[ZobristKey::Piece(
            Piece(ply.by.piece_type, ply.by.color),
            board.pos_to_idx(ply.move_to.from) as u32,
        )
        .to_index()];
        // add new position for moving piece
        hash ^= self.table[ZobristKey::Piece(
            Piece(ply.by.piece_type, ply.by.color),
            board.pos_to_idx(ply.move_to.to) as u32,
        )
        .to_index()];
        // remove captured piece position
        if let Some(captured) = ply.capturing {
            hash ^= self.table[ZobristKey::Piece(
                Piece(captured.piece_type, ply.by.color),
                board.pos_to_idx(captured.pos) as u32,
            )
            .to_index()];
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::chess_engine::{
        moves::{MoveTo, Pos},
        pieces::{PieceColor, PieceType, WHITE_KNIGHT},
    };

    use super::*;

    #[test]
    fn hash_on_default() {
        let board = Game::default();

        assert_ne!(board.zobrist_hash, 0.into());
    }

    #[test]
    fn hash_same_seed() {
        let board = Game::default();
        let other_board = Game::default();

        assert_eq!(board.zobrist_hash, other_board.zobrist_hash);
    }

    #[test]
    fn hash_updates_mailbox() {
        let mut board = Game::default();
        let ply = crate::chess_engine::moves::LegacyPly {
            move_to: MoveTo {
                from: Pos::new(7, 1),
                to: Pos::new(5, 0),
            },
            by: LegacyPiece {
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
    fn hash_updates_bitboard() {
        let mut board = Game::default().boards;
        let ply = crate::chess_engine::bitboard::Ply {
            moving_piece: WHITE_KNIGHT,
            from: 113.into(),
            to: 80.into(),
            ..Default::default()
        };
        let hash_before = board.zobrist_hash;
        board.make_ply(&ply);
        let hash_after = board.zobrist_hash;

        assert_ne!(hash_before, hash_after);
    }

    #[test]
    fn hash_rewinds_mailbox() {
        let mut board = Game::default();
        let ply = crate::chess_engine::moves::LegacyPly {
            move_to: MoveTo {
                from: Pos::new(7, 1),
                to: Pos::new(5, 0),
            },
            by: LegacyPiece {
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

    #[test]
    fn exhaustive_key_iteration() {
        let mut set = HashSet::new();
        for piece in Piece::iter() {
            for i in 0..256 {
                let index = ZobristKey::Piece(piece, i).to_index();
                assert!(index < ZOBRIST_TABLE_LENGTH);
                assert!(set.insert(index));
            }
        }
        let index = ZobristKey::ChangePlayer.to_index();
        assert!(index < ZOBRIST_TABLE_LENGTH);
        assert!(set.insert(index));
        assert_eq!(set.len(), ZOBRIST_TABLE_LENGTH);
    }
}
