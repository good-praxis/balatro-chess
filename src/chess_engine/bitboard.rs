use bevy::prelude::*;
use std::fmt::Display;
use strum::IntoEnumIterator;

use super::pieces::{PieceColor, PieceType};

#[derive(Clone, Debug, Deref, DerefMut)]
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

impl Bitboard {
    #[inline]
    pub fn set(&mut self, index: u32, value: bool) {
        self.0 = self.0 & !(1 << index);
        self.0 = self.0 | ((value as u128) << index);
    }

    #[inline]
    pub fn get(&self, index: u32) -> bool {
        self.0 & (1 << index) != 0
    }
}

#[derive(Debug, Clone)]
pub struct Bitboards {
    /// index = PieceType + (PieceColor * amount of PieceType)
    pub boards: Vec<Bitboard>,
    /// constrains from board size
    limits: Bitboard,
}

impl Bitboards {
    pub fn from_str(input: &str) -> Self {
        let bitboard_count = PieceType::iter().count() * PieceColor::iter().count();
        let mut boards = vec![Bitboard(0u128); bitboard_count];
        let mut limits = Bitboard(0u128);
        let mut idx = 0;
        let mut since_newline: u32 = 0;
        for char in input.trim().chars() {
            // board limit
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
            limits.set(idx, true);

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
            boards[bitboard_idx(piece_type, color)].set(idx, true);

            // increment index
            idx += 1;
        }

        dbg!(limits.to_string());

        Self { boards, limits }
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
    }
}
