use ethnum::u256;

use crate::chess_engine::{
    bitboard::Bitboard,
    pieces::{Piece, PieceWithBitboard},
};

use super::ply::Ply;

const KNIGHT_DIRS: [fn(&Bitboard) -> Bitboard; 8] = [
    Bitboard::shift_nww,
    Bitboard::shift_nnw,
    Bitboard::shift_nne,
    Bitboard::shift_nee,
    Bitboard::shift_see,
    Bitboard::shift_sse,
    Bitboard::shift_ssw,
    Bitboard::shift_sww,
];

impl Bitboard {
    /// Cumulative pseudolegal mask of knight moves
    pub fn knight_move_mask(&self, blocked: &Self, _capturable: &Self) -> Self {
        self.knight_move_arr(blocked, _capturable)
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap_or(Self(u256::ZERO))
    }

    /// Pseudolegal moves by knight
    pub fn knight_move_arr(&self, blocked: &Self, _capturable: &Self) -> Vec<Self> {
        self.shift_in_dirs(&KNIGHT_DIRS, blocked, _capturable)
    }

    /// Mask of threatened positions
    pub fn knight_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.knight_move_mask(blocked, capturable)
    }

    pub fn knight_plys_iter(
        &self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        self.single_step_plys_in_dirs(&KNIGHT_DIRS, blocked, capturable, bitboard_ptr, piece)
    }

    pub fn knight_plys<T: Default + FromIterator<Ply>>(
        &self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> T {
        self.knight_plys_iter(blocked, capturable, bitboard_ptr, piece)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{PieceColor, WHITE_KNIGHT},
    };

    #[test]
    fn knight_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            00000
            0000p
            00n00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_KNIGHT)];

        let expected = Bitboards::from_str(
            r#"
            0n0n0
            n0000
            00000
            n000n
            0n0n0
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_KNIGHT)];
        let mask = board.knight_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn knight_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            00000
            00000
            00n00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_KNIGHT)];

        let expected = Bitboards::from_str(
            r#"
            0n0n0
            n000n
            00000
            n000n
            0n0n0
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_KNIGHT)];
        let mask = board.knight_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn knight_plys() {
        let boards = Bitboards::from_str(
            r#"
            00000
            0000P
            00n00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_KNIGHT)];
        let boards_ptr = boards.boards.as_ptr();

        let mut plys: BinaryHeap<Ply> = board.knight_plys(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
            boards_ptr,
            WHITE_KNIGHT,
        );
        assert_eq!(plys.len(), 8);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
