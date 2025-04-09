use std::collections::BinaryHeap;

use crate::chess_engine::{
    bitboard::Bitboard,
    pieces::{PieceColor, PieceType},
};

use super::ply::Ply;

pub(crate) const KING_DIRS: [fn(&Bitboard) -> Bitboard; 8] = [
    Bitboard::shift_we,
    Bitboard::shift_nw,
    Bitboard::shift_no,
    Bitboard::shift_ne,
    Bitboard::shift_ea,
    Bitboard::shift_se,
    Bitboard::shift_so,
    Bitboard::shift_sw,
];

impl Bitboard {
    /// Cumulative pseudolegal mask of king moves (no castling)
    pub fn king_move_mask(&self, blocked: &Self, _capturable: &Self) -> Self {
        self.king_move_arr(blocked, _capturable)
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap_or(0.into())
    }

    /// Pseudolegal moves by king
    pub fn king_move_arr(&self, blocked: &Self, _capturable: &Self) -> Vec<Self> {
        self.shift_in_dirs(&KING_DIRS, blocked, _capturable)
    }

    /// Mask of threatened positions
    pub fn king_en_prise_mask(&self, blocked: &Self, _capturable: &Self) -> Self {
        self.king_move_mask(blocked, _capturable)
    }

    pub fn king_plys(
        &self,
        blocked: &Self,
        capturable: &Self,
        capturable_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        piece: (PieceType, PieceColor),
    ) -> BinaryHeap<Ply> {
        self.single_step_plys_in_dirs(&KING_DIRS, blocked, capturable, capturable_iter, piece)
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
        pieces::{PieceColor, PieceType},
    };

    #[test]
    fn king_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            00r
            0k0
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            kk0
            k0k
            kkk
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::King, PieceColor::White)];
        let mask = board.king_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn king_move_mask_corner() {
        let boards = Bitboards::from_str(
            r#"
            k0
            00
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            0k
            kk
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::King, PieceColor::White)];
        let mask = board.king_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn king_move_arr_corner() {
        let boards = Bitboards::from_str(
            r#"
            k0
            00
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];

        let arr = board.king_move_arr(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn king_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            000
            0k0
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            kkk
            k0k
            kkk
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::King, PieceColor::White)];
        let mask = board.king_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn king_plys() {
        let boards = Bitboards::from_str(
            r#"
            00P
            0k0
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];

        let mut plys = board.king_plys(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
            boards.all_piece_types_by_color(PieceColor::Black),
            (PieceType::King, PieceColor::White),
        );
        assert_eq!(plys.len(), 8);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
