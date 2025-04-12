use crate::chess_engine::{
    bitboard::Bitboard,
    pieces::{PieceColor, PieceType},
};

use super::ply::Ply;

const ROOK_STEP_DIRS: [fn(&Bitboard, &Bitboard, &Bitboard) -> Vec<Bitboard>; 4] = [
    Bitboard::step_we,
    Bitboard::step_no,
    Bitboard::step_ea,
    Bitboard::step_so,
];

impl Bitboard {
    /// Cumulative pseudolegal  mask of rook moves (no castling)
    pub fn rook_move_mask(&self, blocked: &Bitboard, capturable: &Bitboard) -> Self {
        let dirs = [Self::fill_we, Self::fill_no, Self::fill_ea, Self::fill_so];
        self.fill_in_dirs(&dirs, blocked, capturable)
    }

    /// Pseudolegal moves by rook
    pub fn rook_move_arr(&self, blocked: &Bitboard, capturable: &Bitboard) -> Vec<Self> {
        self.step_in_dirs(&ROOK_STEP_DIRS, blocked, capturable)
    }

    /// Mask of threatened positions
    pub fn rook_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.rook_move_mask(blocked, capturable)
    }

    pub fn rook_plys_iter(
        &self,
        blocked: &Self,
        capturable: &Self,
        capturable_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        piece: (PieceType, PieceColor),
    ) -> impl Iterator<Item = Ply> {
        self.multi_step_plys_in_dirs(&ROOK_STEP_DIRS, blocked, capturable, capturable_iter, piece)
    }

    pub fn rook_plys<T: Default + FromIterator<Ply>>(
        &self,
        blocked: &Self,
        capturable: &Self,
        capturable_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        piece: (PieceType, PieceColor),
    ) -> T {
        self.rook_plys_iter(blocked, capturable, capturable_iter, piece)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{PieceColor, PieceType},
    };

    #[test]
    fn rook_move_arr() {
        let boards = Bitboards::from_str(
            r#"
            00p00
            00000
            00r0P
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];

        let arr = board.rook_move_arr(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(arr.len(), 7);
    }

    #[test]
    fn rook_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            00000
            00000
            00r0P
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            00r00
            00r00
            rr0rr
            00r00
            00r00
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];
        let mask = board.rook_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn rook_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            00000
            00000
            00r0P
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            00r00
            00r00
            rr0rr
            00r00
            00r00
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];
        let mask = board.rook_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn rook_plys() {
        let boards = Bitboards::from_str(
            r#"
            00000
            00000
            00r0P
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];

        let mut plys: BinaryHeap<Ply> = board.rook_plys(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
            boards.all_piece_types_by_color(PieceColor::Black),
            (PieceType::Rook, PieceColor::White),
        );
        assert_eq!(plys.len(), 8);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
