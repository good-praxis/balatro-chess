use bevy::prelude::*;
use move_gen::ply::legality_filter;
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};
use strum::IntoEnumIterator;

use super::{
    pieces::{PieceColor, PieceType},
    zobrist::{Zobrist, ZobristHash},
};

pub mod bitwise_traits;
pub mod move_gen;

mod search;

pub use move_gen::ply::Ply;

/// u32 based position on the Bitboard. Derived by couting `trailing_zeros`
#[derive(Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq, Copy)]
pub struct BitIndex(u32);

impl From<u32> for BitIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Bitboard> for BitIndex {
    #[inline]
    fn from(value: Bitboard) -> Self {
        value.to_bit_idx()
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq, Copy)]
pub struct Bitboard(u128);

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board_str = String::new();
        let mut copy = self.0;
        for _ in 0..16 {
            for _ in 0..16 {
                if copy & 1 == 0 {
                    board_str.push('.');
                } else {
                    board_str.push('1');
                }
                copy = copy.wrapping_shr(1);
            }
            board_str.push('\n');
        }
        write!(f, "{}", board_str)
    }
}

impl From<BitIndex> for Bitboard {
    #[inline]
    fn from(value: BitIndex) -> Self {
        Self(1 << *value)
    }
}

impl From<u128> for Bitboard {
    #[inline]
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl Bitboard {
    #[inline]
    pub fn set(&mut self, index: BitIndex, value: bool) {
        *self &= !(1 << *index);
        *self |= (value as u128) << *index;
    }

    #[inline]
    pub fn get<T: std::ops::Deref<Target = u32>>(&self, index: T) -> bool {
        **self & (1 << *index) != 0
    }

    /// Gets the position for the
    #[inline]
    pub fn to_bit_idx(&self) -> BitIndex {
        self.trailing_zeros().into()
    }
}

#[derive(Debug, Clone)]
pub struct Bitboards {
    /// index = PieceType + (PieceColor * amount of PieceType)
    pub boards: Vec<Bitboard>,
    pub piece_list: Vec<Vec<BitIndex>>,
    /// constrains from board size, 1 = active tile;
    limits: Bitboard,
    /// mask of all pieces in their initial position.
    /// updated on moves or captures
    unmoved_pieces: Bitboard,
    /// Board of en passant vulnerable positions
    en_passant: Bitboard,

    // Zobrist hashing
    pub zobrist_table: Arc<Zobrist>,
    pub zobrist_hash: ZobristHash,

    // thricefold repetition protection
    pub visited_positions: Arc<Mutex<HashMap<ZobristHash, isize>>>,
}

impl PartialEq for Bitboards {
    fn eq(&self, other: &Self) -> bool {
        self.zobrist_hash == other.zobrist_hash
    }
}

impl Bitboards {
    pub fn from_str(input: &str) -> Self {
        let bitboard_count = PieceType::iter().count() * PieceColor::iter().count();
        let mut boards = vec![Bitboard(0u128); bitboard_count];
        let mut piece_list = vec![vec![]; bitboard_count];
        let mut limits = Bitboard(0u128);
        let mut idx = 0;
        let mut since_newline: u32 = 0;
        for char in input.trim().chars() {
            if char == '\n' {
                let delta = since_newline.abs_diff(16);
                idx += delta;
                since_newline = 0;
                continue;
            }
            // any other whitespace
            if char.is_whitespace() {
                continue;
            }
            if since_newline >= 16 {
                panic!("Board too wide! Size of 16x16 is the limit");
            }
            since_newline += 1;
            limits.set(idx.into(), true);

            // Empty square
            if char == '0' {
                idx += 1;
                continue;
            }

            // Determine color and type
            let color = if char.is_ascii_lowercase() {
                PieceColor::White
            } else {
                PieceColor::Black
            };
            let piece_type = match char {
                'k' | 'K' => PieceType::King,
                'q' | 'Q' => PieceType::Queen,
                'r' | 'R' => PieceType::Rook,
                'n' | 'N' => PieceType::Knight,
                'b' | 'B' => PieceType::Bishop,
                'p' | 'P' => PieceType::Pawn,
                _ => panic!("Unexpected char: {}", char),
            };

            // flip bit in question
            boards[bitboard_idx(piece_type, color)].set(idx.into(), true);

            // update piece_list with piece
            piece_list[bitboard_idx(piece_type, color)].push(idx.into());

            // increment index
            idx += 1;
        }

        let unmoved_pieces = boards.iter().fold(Bitboard(0), |acc, e| acc | *e);

        let zobrist_table = Arc::new(Zobrist::new(128));

        let mut new_bitboards = Self {
            boards,
            piece_list,
            limits,
            unmoved_pieces,
            en_passant: Bitboard(0),
            zobrist_table,
            zobrist_hash: 0.into(),
            visited_positions: Arc::new(Mutex::new(HashMap::new())),
        };

        let zobrist_hash = new_bitboards
            .zobrist_table
            .gen_initial_hash_bitboard(new_bitboards.key_value_pieces_iter());
        new_bitboards.zobrist_hash = zobrist_hash;
        new_bitboards
            .visited_positions
            .lock()
            .unwrap()
            .insert(zobrist_hash, 1);
        new_bitboards
    }

    pub fn key_value_pieces_iter(
        &self,
    ) -> impl Iterator<Item = ((PieceType, PieceColor), BitIndex)> {
        PieceType::iter()
            .map(|piece_type| {
                return [
                    (piece_type, PieceColor::White),
                    (piece_type, PieceColor::Black),
                ];
            })
            .flatten()
            .map(|(piece_type, piece_color)| {
                let bitboard_idx = bitboard_idx(piece_type, piece_color);
                self.piece_list[bitboard_idx]
                    .iter()
                    .map(move |idx| ((piece_type, piece_color), *idx))
            })
            .flatten()
    }

    pub fn all_pieces(&self) -> Bitboard {
        self.boards.iter().fold(Bitboard(0), |acc, e| acc | *e)
    }

    pub fn all_pieces_by_color(&self, color: PieceColor) -> Bitboard {
        let mut board = Bitboard(0);
        for piece_type in PieceType::iter() {
            board |= self.boards[bitboard_idx(piece_type, color)];
        }
        board
    }

    /// Primarily used when we don't want a full mask of all pieces, but want to determine which piece we are capturing
    pub fn all_piece_types_by_color(
        &self,
        color: PieceColor,
    ) -> impl Iterator<Item = (PieceType, Bitboard)> + Clone {
        PieceType::iter()
            .map(move |piece_type| (piece_type, self.boards[bitboard_idx(piece_type, color)]))
    }

    /// Used with functions asked for blocking masks
    pub fn blocked_mask_for_color(&self, color: PieceColor) -> Bitboard {
        !self.limits | self.all_pieces_by_color(color)
    }

    pub fn en_prise_by_color(&self, color: PieceColor) -> Bitboard {
        let mut board = Bitboard(0);
        for piece_type in PieceType::iter() {
            for idx in self.piece_list[bitboard_idx(piece_type, color)].clone() {
                board |= match piece_type {
                    PieceType::King => Bitboard::from(idx).king_en_prise_mask(
                        &self.blocked_mask_for_color(color),
                        &self.all_pieces_by_color(color.next()),
                    ),
                    PieceType::Queen => Bitboard::from(idx).queen_en_prise_mask(
                        &self.blocked_mask_for_color(color),
                        &self.all_pieces_by_color(color.next()),
                    ),
                    PieceType::Rook => Bitboard::from(idx).rook_en_prise_mask(
                        &self.blocked_mask_for_color(color),
                        &self.all_pieces_by_color(color.next()),
                    ),
                    PieceType::Bishop => Bitboard::from(idx).bishop_en_prise_mask(
                        &self.blocked_mask_for_color(color),
                        &self.all_pieces_by_color(color.next()),
                    ),
                    PieceType::Knight => Bitboard::from(idx).knight_en_prise_mask(
                        &self.blocked_mask_for_color(color),
                        &self.all_pieces_by_color(color.next()),
                    ),
                    PieceType::Pawn => Bitboard::from(idx)
                        .pawn_en_prise_mask(&self.blocked_mask_for_color(color), color),
                }
            }
        }
        board
    }

    /// all legal plys by color
    pub fn all_legal_plys_by_color<T: Default + Extend<Ply>>(&self, color: PieceColor) -> T {
        PieceType::iter().fold(Default::default(), |mut coll, piece_type| {
            for piece in self.piece_list[bitboard_idx(piece_type, color)].iter() {
                let board = Bitboard::from(*piece);
                let blocked = &self.blocked_mask_for_color(color);
                let capturable = &self.all_pieces_by_color(color.next());
                let capturable_iter = self.all_piece_types_by_color(color.next());
                let piece = (piece_type, color);
                match piece_type {
                    PieceType::King => {
                        coll.extend(legality_filter(
                            board.king_plys_iter(blocked, capturable, capturable_iter, piece),
                            self,
                        ));
                    }
                    PieceType::Queen => {
                        coll.extend(legality_filter(
                            board.queen_plys_iter(blocked, capturable, capturable_iter, piece),
                            self,
                        ));
                    }
                    PieceType::Rook => {
                        coll.extend(legality_filter(
                            board.rook_plys_iter(blocked, capturable, capturable_iter, piece),
                            self,
                        ));
                    }

                    PieceType::Bishop => {
                        coll.extend(legality_filter(
                            board.bishop_plys_iter(blocked, capturable, capturable_iter, piece),
                            self,
                        ));
                    }
                    PieceType::Knight => {
                        coll.extend(legality_filter(
                            board.knight_plys_iter(blocked, capturable, capturable_iter, piece),
                            self,
                        ));
                    }

                    PieceType::Pawn => coll.extend(legality_filter(
                        board.pawn_plys_iter(
                            blocked,
                            capturable,
                            capturable_iter,
                            color,
                            &self.unmoved_pieces,
                            &self.en_passant,
                        ),
                        self,
                    )),
                };
            }
            coll
        })
    }
}

/// Bitboard index of a certain PieceType and PieceColor combo
pub fn bitboard_idx(piece_type: PieceType, piece_color: PieceColor) -> usize {
    piece_type as usize + (piece_color as usize * PieceType::iter().count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess_engine::game::Game;

    #[test]
    fn bitboard_getter() {
        let bitboard = Bitboard(0b01);

        assert!(bitboard.get(&0));
        assert!(!bitboard.get(&1));
    }

    #[test]
    fn bitboard_setter() {
        let mut bitboard = Bitboard(0b01);
        bitboard.set(0.into(), false);
        bitboard.set(1.into(), true);

        assert_eq!(*bitboard, 0b10);
    }

    #[test]
    fn limits_default_amount() {
        let game = Game::default();

        assert_eq!(game.boards.limits.count_ones(), 64);
    }

    #[test]
    fn limits_amount_from_string() {
        let bitboards = Bitboards::from_str(
            r#"
        RK00
        0000
        rkr0
        "#,
        );
        assert_eq!(bitboards.limits.count_ones(), 12);
    }

    #[test]
    fn expected_piece_counts_default() {
        use PieceColor::*;
        use PieceType::*;
        let game = Game::default();
        let bb = game.boards.boards;
        assert_eq!(bb[bitboard_idx(King, White)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(King, Black)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(Queen, White)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(Queen, Black)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(Rook, White)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(Rook, Black)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(Knight, White)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(Knight, Black)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(Bishop, White)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(Bishop, Black)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(Pawn, White)].count_ones(), 8);
        assert_eq!(bb[bitboard_idx(Pawn, Black)].count_ones(), 8);

        let all_pieces = bb.into_iter().reduce(|acc, e| acc | e).unwrap();
        assert_eq!(all_pieces.count_ones(), 32);
    }

    #[test]
    fn expected_piece_list_default() {
        use PieceColor::*;
        use PieceType::*;
        let game = Game::default();
        let p = game.boards.piece_list;
        assert_eq!(p[bitboard_idx(King, White)].len(), 1);
        assert_eq!(p[bitboard_idx(King, Black)].len(), 1);
        assert_eq!(p[bitboard_idx(Queen, White)].len(), 1);
        assert_eq!(p[bitboard_idx(Queen, Black)].len(), 1);
        assert_eq!(p[bitboard_idx(Rook, White)].len(), 2);
        assert_eq!(p[bitboard_idx(Rook, Black)].len(), 2);
        assert_eq!(p[bitboard_idx(Knight, White)].len(), 2);
        assert_eq!(p[bitboard_idx(Knight, Black)].len(), 2);
        assert_eq!(p[bitboard_idx(Bishop, White)].len(), 2);
        assert_eq!(p[bitboard_idx(Bishop, Black)].len(), 2);
        assert_eq!(p[bitboard_idx(Pawn, White)].len(), 8);
        assert_eq!(p[bitboard_idx(Pawn, Black)].len(), 8);
    }

    #[test]
    fn all_pieces() {
        let game = Game::default();
        let bitboards = game.boards;

        let all_pieces = bitboards.all_pieces();
        assert_eq!(all_pieces.count_ones(), 32);
    }

    #[test]
    fn all_pieces_by_color() {
        let game = Game::default();
        let bitboards = game.boards;

        let white_pieces = bitboards.all_pieces_by_color(PieceColor::White);
        let black_pieces = bitboards.all_pieces_by_color(PieceColor::Black);
        assert_eq!(white_pieces.count_ones(), 16);
        assert_eq!(black_pieces.count_ones(), 16);
        assert_eq!(white_pieces & black_pieces, 0.into());
    }

    #[test]
    fn bitboard_from_bit_idx() {
        let bitboard: Bitboard = BitIndex(3).into();
        assert_eq!(bitboard, Bitboard(8));
    }

    #[test]
    fn bitboard_to_bit_idx() {
        let mut bitboard = Bitboard(0);
        bitboard.set(30.into(), true);
        assert_eq!(bitboard.to_bit_idx(), 30.into());
    }

    #[test]
    fn en_prise_default() {
        let game = Game::default();
        let bitboard = game.boards;

        let expected = Bitboards::from_str(
            r#"
        00000000
        00000000
        pppppppp
        00000000
        00000000
        pppppppp
        00000000
        00000000
        "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];

        let en_prise = bitboard.en_prise_by_color(PieceColor::White)
            | bitboard.en_prise_by_color(PieceColor::Black);
        assert_eq!(en_prise, expected);
    }

    #[test]
    fn all_moves_by_sites_default() {
        let game = Game::default();
        let boards = game.boards;
        let white_moves: Vec<Ply> = boards.all_legal_plys_by_color(PieceColor::White);
        assert_eq!(white_moves.len(), 20);
        let black_moves: Vec<Ply> = boards.all_legal_plys_by_color(PieceColor::Black);
        assert_eq!(black_moves.len(), 20);
    }

    #[test]
    fn all_moves_by_sites_complex() {
        let boards = Bitboards::from_str(
            r#"
        00000
        00k00
        00rB0
        p000b
        00000
        "#,
        );
        let white_moves: Vec<Ply> = boards.all_legal_plys_by_color(PieceColor::White);
        assert_eq!(white_moves.len(), 8);
    }
}
