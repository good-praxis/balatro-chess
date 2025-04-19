use bevy::prelude::*;
use ethnum::u256;
use move_gen::ply::{captures_only, legality_filter};
use simplehash::FnvHasher64;
use std::{
    collections::HashMap,
    fmt::Display,
    hash::BuildHasherDefault,
    sync::{Arc, Mutex},
};
use strum::IntoEnumIterator;

use super::{
    pieces::{
        PIECE_COMBO_COUNT, PIECE_TYPE_COUNT, Piece, PieceColor, PieceType, PieceWithBitboard,
    },
    zobrist::{Zobrist, ZobristHash},
};

pub mod bitwise_traits;
pub mod move_gen;

mod search;
pub use search::Weights;

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

impl Display for BitIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We know that the board will grow in row length to the 'right',
        // i*16 will always be the first used file, and therefore always be 'A'
        // so we can always know the file label by it's `x % 16` result
        let file = ('A' as u8 + (**self % 16) as u8) as char;

        // We flip convention here, defining rank to be counted from top
        // so that we can naturally grow the board 'downwards'. This lets us
        // omit counting the active rows; Which require a reference to the
        // limit board.
        let rank = **self / 16 + 1;

        f.write_fmt(format_args!("{}{}", file, rank))
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq, Copy)]
pub struct Bitboard(u256);

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
        Self(u256::from_words(0, 1) << *value)
    }
}

impl From<u256> for Bitboard {
    #[inline]
    fn from(value: u256) -> Self {
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

    /// Reduce bitboard to a column-wise representation by or-ing 16-bit words
    pub fn to_column_representation(&self) -> u16 {
        let bytes = self.to_be_bytes();
        let mut words = [0u16; 16];
        for i in 0..16 {
            let offset = i * 2;
            words[i] = bytes[offset] as u16;
            words[i] <<= 8;
            words[i] += bytes[offset + 1] as u16;
        }

        words.iter().fold(0, |acc, e| acc | e)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Bitboards {
    /// index = PieceType + (PieceColor * amount of PieceType)
    pub boards: [Bitboard; PIECE_COMBO_COUNT],

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

    //`FnvHasher64` has proven to be the most efficient in testing for these HashMaps
    /// thricefold repetition protection.
    pub visited_positions: Arc<Mutex<HashMap<u32, isize, BuildHasherDefault<FnvHasher64>>>>,

    // Search-related lookup tables
    /// if false we don't need to lock the mutex
    pub check_cache: bool,
    /// Storing
    pub quiescence_table: Arc<Mutex<HashMap<(u32, u16, u8), i32, BuildHasherDefault<FnvHasher64>>>>,
    pub pv_table: Arc<Mutex<HashMap<(u32, u16), Ply, BuildHasherDefault<FnvHasher64>>>>,
    //pub evaluation_table: Arc<Mutex<HashMap<u32, i32, BuildHasherDefault<FnvHasher64>>>>,
    pub en_prise_table: Arc<Mutex<HashMap<(u32, u8), Bitboard, BuildHasherDefault<FnvHasher64>>>>,
}

impl PartialEq for Bitboards {
    fn eq(&self, other: &Self) -> bool {
        self.zobrist_hash == other.zobrist_hash
    }
}

impl Display for Bitboards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mailbox = self.to_mailbox();
        let mut board_str = " ".to_string();
        for i in 0..self.limits.trailing_ones() {
            board_str.push(('A' as u8 + i as u8) as char);
        }

        for (i, piece) in mailbox.iter().enumerate() {
            let rank = ('1' as u8 + (i as u32 / self.limits.trailing_ones()) as u8) as char;
            if i as u32 % self.limits.trailing_ones() == 0 {
                board_str.push('\n');
                board_str.push(rank);
            }
            if let Some(piece) = piece {
                board_str.push(piece.to_char());
            } else {
                board_str.push_str("-");
            }
        }
        write!(f, "{}", board_str)
    }
}

impl Bitboards {
    pub fn from_str(input: &str) -> Self {
        let mut boards = [Bitboard(u256::ZERO); PIECE_COMBO_COUNT];
        let mut piece_list = vec![vec![]; PIECE_COMBO_COUNT];
        let mut limits = Bitboard(u256::ZERO);
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

            let piece: Piece = char.into();

            // flip bit in question
            boards[bitboard_idx(piece)].set(idx.into(), true);

            // update piece_list with piece
            piece_list[bitboard_idx(piece)].push(idx.into());

            // increment index
            idx += 1;
        }

        let unmoved_pieces = boards.iter().fold(Bitboard(u256::ZERO), |acc, e| acc | *e);

        let zobrist_table = Arc::new(Zobrist::new());

        let mut new_bitboards = Self {
            boards,
            piece_list,
            limits,
            unmoved_pieces,
            zobrist_table,
            ..Default::default()
        };

        let zobrist_hash = new_bitboards
            .zobrist_table
            .gen_initial_hash_bitboard(new_bitboards.key_value_pieces_iter());
        new_bitboards.zobrist_hash = zobrist_hash;
        new_bitboards
            .visited_positions
            .lock()
            .unwrap()
            .insert(*zobrist_hash, 1);
        new_bitboards
    }

    pub fn to_mailbox(&self) -> Vec<Option<Piece>> {
        let tile_count = self.limits.count_ones() as usize;
        let mut mailbox = vec![None; tile_count];
        let row_length = self.limits.trailing_ones();

        for piece in Piece::iter() {
            let bitboard_idx = bitboard_idx(piece);
            for pos in self.piece_list[bitboard_idx].iter() {
                let mailbox_idx = (**pos % 16 + (row_length * (**pos / 16))) as usize;
                mailbox[mailbox_idx] = Some(piece);
            }
        }

        mailbox
    }

    pub fn key_value_pieces_iter(&self) -> impl Iterator<Item = (Piece, BitIndex)> {
        Piece::iter().flat_map(|piece| {
            let bitboard_idx = bitboard_idx(piece);
            self.piece_list[bitboard_idx]
                .iter()
                .map(move |idx| (piece, *idx))
        })
    }

    pub fn all_pieces(&self) -> Bitboard {
        self.boards
            .iter()
            .fold(Bitboard(u256::ZERO), |acc, e| acc | *e)
    }

    pub fn all_pieces_by_color(&self, color: PieceColor) -> Bitboard {
        let mut board = Bitboard(u256::ZERO);
        for piece in Piece::iter_color(color) {
            board |= self.boards[bitboard_idx(piece)];
        }
        board
    }

    /// Used with functions asked for blocking masks
    pub fn blocked_mask_for_color(&self, color: PieceColor) -> Bitboard {
        !self.limits | self.all_pieces_by_color(color)
    }

    ///
    pub fn en_prise_by_color(&self, color: PieceColor) -> Bitboard {
        let mut en_prise_table = self.en_prise_table.lock().unwrap();
        if let Some(en_prise) = en_prise_table.get(&(*self.zobrist_hash, color as u8)) {
            return *en_prise;
        }

        let mut board = Bitboard(u256::ZERO);
        for piece in Piece::iter_color(color) {
            for i in 0..self.piece_list[bitboard_idx(piece)].len() {
                let idx = self.piece_list[bitboard_idx(piece)][i];
                board |= match piece.0 {
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
        en_prise_table.insert((*self.zobrist_hash, color as u8), board);
        board
    }

    /// all legal plys by color
    pub fn all_legal_plys_by_color<T: Default + Extend<Ply>>(&mut self, color: PieceColor) -> T {
        PieceType::iter().fold(Default::default(), |mut coll, piece_type| {
            for i in 0..self.piece_list[bitboard_idx(Piece(piece_type, color))].len() {
                let piece = self.piece_list[bitboard_idx(Piece(piece_type, color))][i];
                let board = Bitboard::from(piece);
                let blocked = &self.blocked_mask_for_color(color);
                let capturable = &self.all_pieces_by_color(color.next());
                let bitboard_ptr = self.boards.as_ptr();
                let piece = Piece(piece_type, color);
                match piece_type {
                    PieceType::King => {
                        coll.extend(legality_filter(
                            board.king_plys_iter(blocked, capturable, bitboard_ptr, piece),
                            self,
                        ));
                    }
                    PieceType::Queen => {
                        coll.extend(legality_filter(
                            board.queen_plys_iter(blocked, capturable, bitboard_ptr, piece),
                            self,
                        ));
                    }
                    PieceType::Rook => {
                        coll.extend(legality_filter(
                            board.rook_plys_iter(blocked, capturable, bitboard_ptr, piece),
                            self,
                        ));
                    }

                    PieceType::Bishop => {
                        coll.extend(legality_filter(
                            board.bishop_plys_iter(blocked, capturable, bitboard_ptr, piece),
                            self,
                        ));
                    }
                    PieceType::Knight => {
                        coll.extend(legality_filter(
                            board.knight_plys_iter(blocked, capturable, bitboard_ptr, piece),
                            self,
                        ));
                    }

                    PieceType::Pawn => coll.extend(legality_filter(
                        board.pawn_plys_iter(
                            blocked,
                            capturable,
                            bitboard_ptr,
                            color,
                            &raw const self.unmoved_pieces,
                            &raw const self.en_passant,
                        ),
                        self,
                    )),
                };
            }
            coll
        })
    }

    /// all legal capturing_plys by color
    pub fn all_legal_capturing_plys_by_color<T: Default + Extend<Ply>>(
        &mut self,
        color: PieceColor,
    ) -> T {
        PieceType::iter().fold(Default::default(), |mut coll, piece_type| {
            for i in 0..self.piece_list[bitboard_idx(Piece(piece_type, color))].len() {
                let piece = self.piece_list[bitboard_idx(Piece(piece_type, color))][i];
                let board = Bitboard::from(piece);
                let blocked = &self.blocked_mask_for_color(color);
                let capturable = &self.all_pieces_by_color(color.next());
                let bitboards_ptr = self.boards.as_ptr();
                let piece = Piece(piece_type, color);
                match piece_type {
                    PieceType::King => {
                        coll.extend(legality_filter(
                            captures_only(board.king_plys_iter(
                                blocked,
                                capturable,
                                bitboards_ptr,
                                piece,
                            )),
                            self,
                        ));
                    }
                    PieceType::Queen => {
                        coll.extend(legality_filter(
                            captures_only(board.queen_plys_iter(
                                blocked,
                                capturable,
                                bitboards_ptr,
                                piece,
                            )),
                            self,
                        ));
                    }
                    PieceType::Rook => {
                        coll.extend(legality_filter(
                            captures_only(board.rook_plys_iter(
                                blocked,
                                capturable,
                                bitboards_ptr,
                                piece,
                            )),
                            self,
                        ));
                    }

                    PieceType::Bishop => {
                        coll.extend(legality_filter(
                            captures_only(board.bishop_plys_iter(
                                blocked,
                                capturable,
                                bitboards_ptr,
                                piece,
                            )),
                            self,
                        ));
                    }
                    PieceType::Knight => {
                        coll.extend(legality_filter(
                            captures_only(board.knight_plys_iter(
                                blocked,
                                capturable,
                                bitboards_ptr,
                                piece,
                            )),
                            self,
                        ));
                    }

                    PieceType::Pawn => coll.extend(legality_filter(
                        captures_only(board.pawn_plys_iter(
                            blocked,
                            capturable,
                            bitboards_ptr,
                            color,
                            &raw const self.unmoved_pieces,
                            &raw const self.en_passant,
                        )),
                        self,
                    )),
                };
            }
            coll
        })
    }
}

/// Primarily used when we don't want a full mask of all pieces, but want to determine which piece we are capturing
pub fn all_pieces_by_color_from_ptr_iter(
    boards: *const Bitboard,
    color: PieceColor,
) -> impl Iterator<Item = PieceWithBitboard> + Clone {
    Piece::iter_color(color)
        .map(move |piece| PieceWithBitboard(piece, unsafe { *boards.add(bitboard_idx(piece)) }))
}

/// Bitboard index of a certain PieceType and PieceColor combo
#[inline]
pub fn bitboard_idx(piece: Piece) -> usize {
    piece.0 as usize + (piece.1 as usize * PIECE_TYPE_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess_engine::{game::Game, pieces::*};

    #[test]
    fn bitboard_getter() {
        let bitboard = Bitboard(0b01u32.into());

        assert!(bitboard.get(&0));
        assert!(!bitboard.get(&1));
    }

    #[test]
    fn bitboard_setter() {
        let mut bitboard = Bitboard(0b01u32.into());
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
        let game = Game::default();
        let bb = game.boards.boards;
        assert_eq!(bb[bitboard_idx(WHITE_KING)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(BLACK_KING)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(WHITE_QUEEN)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(BLACK_QUEEN)].count_ones(), 1);
        assert_eq!(bb[bitboard_idx(WHITE_ROOK)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(BLACK_ROOK)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(WHITE_KNIGHT)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(BLACK_KNIGHT)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(WHITE_BISHOP)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(BLACK_BISHOP)].count_ones(), 2);
        assert_eq!(bb[bitboard_idx(WHITE_PAWN)].count_ones(), 8);
        assert_eq!(bb[bitboard_idx(BLACK_PAWN)].count_ones(), 8);

        let all_pieces = bb.into_iter().reduce(|acc, e| acc | e).unwrap();
        assert_eq!(all_pieces.count_ones(), 32);
    }

    #[test]
    fn expected_piece_list_default() {
        let game = Game::default();
        let p = game.boards.piece_list;
        assert_eq!(p[bitboard_idx(WHITE_KING)].len(), 1);
        assert_eq!(p[bitboard_idx(BLACK_KING)].len(), 1);
        assert_eq!(p[bitboard_idx(WHITE_QUEEN)].len(), 1);
        assert_eq!(p[bitboard_idx(BLACK_QUEEN)].len(), 1);
        assert_eq!(p[bitboard_idx(WHITE_ROOK)].len(), 2);
        assert_eq!(p[bitboard_idx(BLACK_ROOK)].len(), 2);
        assert_eq!(p[bitboard_idx(WHITE_KNIGHT)].len(), 2);
        assert_eq!(p[bitboard_idx(BLACK_KNIGHT)].len(), 2);
        assert_eq!(p[bitboard_idx(WHITE_BISHOP)].len(), 2);
        assert_eq!(p[bitboard_idx(BLACK_BISHOP)].len(), 2);
        assert_eq!(p[bitboard_idx(WHITE_PAWN)].len(), 8);
        assert_eq!(p[bitboard_idx(BLACK_PAWN)].len(), 8);
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
        assert_eq!(white_pieces & black_pieces, Bitboard(0u32.into()));
    }

    #[test]
    fn bitboard_from_bit_idx() {
        let bitboard: Bitboard = BitIndex(3).into();
        assert_eq!(bitboard, Bitboard(8u32.into()));
    }

    #[test]
    fn bitboard_to_bit_idx() {
        let mut bitboard = Bitboard(0u32.into());
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
        let expected = expected.boards[bitboard_idx(WHITE_PAWN)];

        let en_prise = bitboard.en_prise_by_color(PieceColor::White)
            | bitboard.en_prise_by_color(PieceColor::Black);
        assert_eq!(en_prise, expected);
    }

    #[test]
    fn all_moves_by_sites_default() {
        let game = Game::default();
        let mut boards = game.boards;
        let white_moves: Vec<Ply> = boards.all_legal_plys_by_color(PieceColor::White);
        assert_eq!(white_moves.len(), 20);
        let black_moves: Vec<Ply> = boards.all_legal_plys_by_color(PieceColor::Black);
        assert_eq!(black_moves.len(), 20);
    }

    #[test]
    fn all_moves_by_sites_complex() {
        let mut boards = Bitboards::from_str(
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

    #[test]
    fn all_captures_by_sites_complex() {
        let mut boards = Bitboards::from_str(
            r#"
        00000
        00k00
        00rB0
        p000b
        00000
        "#,
        );
        let white_moves: Vec<Ply> = boards.all_legal_capturing_plys_by_color(PieceColor::White);
        assert_eq!(white_moves.len(), 3);
    }

    #[test]
    fn to_mailbox() {
        let boards = Bitboards::from_str(
            r#"
        p00
        BKk
        QRr

        "#,
        );
        let mailbox = boards.to_mailbox();
        assert_eq!(mailbox.len(), 9);
        assert_eq!(
            mailbox,
            vec![
                Some(WHITE_PAWN),
                None,
                None,
                Some(BLACK_BISHOP),
                Some(BLACK_KING),
                Some(WHITE_KING),
                Some(BLACK_QUEEN),
                Some(BLACK_ROOK),
                Some(WHITE_ROOK),
            ]
        );
    }

    #[test]
    fn test_column_representation() {
        let boards = Bitboards::from_str(
            r#"
        00000p00
        00p00000
        0000p000
        p0000000
        00000000
        000p0000
        "#,
        );
        let expect: u16 = 0b00111101;
        let pawns = boards.boards[bitboard_idx(WHITE_PAWN)];
        let column_rep = pawns.to_column_representation();
        assert_eq!(column_rep, expect);
    }

    #[test]
    fn test_column_representation_full_width() {
        let boards = Bitboards::from_str(
            r#"
        00000p000000p000
        00p000000000000p
        0000p00000000000
        p000000000000000
        0000000000000000
        000p000000000000
        "#,
        );
        let expect: u16 = 0b1001000000111101;
        let pawns = boards.boards[bitboard_idx(WHITE_PAWN)];
        let column_rep = pawns.to_column_representation();
        dbg!(format!("{:b}\n{:b}", column_rep, expect));
        assert_eq!(column_rep, expect);
    }
}
