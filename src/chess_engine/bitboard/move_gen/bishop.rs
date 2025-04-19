use crate::chess_engine::{
    bitboard::Bitboard,
    pieces::{Piece, PieceWithBitboard},
};

use super::ply::Ply;

const BISHOP_STEP_DIRS: [fn(&Bitboard, &Bitboard, &Bitboard) -> Vec<Bitboard>; 4] = [
    Bitboard::step_nw,
    Bitboard::step_ne,
    Bitboard::step_se,
    Bitboard::step_sw,
];

impl Bitboard {
    /// Cumulative pseudolegal mask of bishop moves
    pub fn bishop_move_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        let dirs = [Self::fill_nw, Self::fill_ne, Self::fill_se, Self::fill_sw];
        self.fill_in_dirs(&dirs, blocked, capturable)
    }

    /// Pseudolegal moves by bishop
    pub fn bishop_move_arr(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_in_dirs(&BISHOP_STEP_DIRS, blocked, capturable)
    }

    /// Mask of threatened positions
    pub fn bishop_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.bishop_move_mask(blocked, capturable)
    }

    pub fn bishop_plys_iter(
        &self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        self.multi_step_plys_in_dirs(&BISHOP_STEP_DIRS, blocked, capturable, bitboard_ptr, piece)
    }

    pub fn bishop_plys<T: Default + FromIterator<Ply>>(
        &self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> T {
        self.bishop_plys_iter(blocked, capturable, bitboard_ptr, piece)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{PieceColor, WHITE_BISHOP},
    };

    #[test]
    fn bishop_move_arr() {
        let boards = Bitboards::from_str(
            r#"
            p000P
            00000
            00b00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];

        let arr = board.bishop_move_arr(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(arr.len(), 7);
    }

    #[test]
    fn bishop_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            p000P
            00000
            00b00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];

        let expected = Bitboards::from_str(
            r#"
            0000b
            0b0b0
            00000
            0b0b0
            b000b
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_BISHOP)];
        let mask = board.bishop_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn bishop_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            0000P
            00000
            00b00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];

        let expected = Bitboards::from_str(
            r#"
            b000b
            0b0b0
            00000
            0b0b0
            b000b
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_BISHOP)];
        let mask = board.bishop_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn bishop_plys() {
        let boards = Bitboards::from_str(
            r#"
            0000P
            00000
            00b00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];

        let mut plys: BinaryHeap<Ply> = board.bishop_plys(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
            boards.boards.as_ptr(),
            WHITE_BISHOP,
        );
        assert_eq!(plys.len(), 8);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
