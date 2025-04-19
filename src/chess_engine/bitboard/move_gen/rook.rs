use crate::chess_engine::{bitboard::Bitboard, pieces::Piece};

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

    /// Mask of threatened positions
    pub fn rook_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.rook_move_mask(blocked, capturable)
    }

    pub fn rook_plys(
        &self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        self.multi_step_plys_in_dirs(&ROOK_STEP_DIRS, blocked, capturable, bitboard_ptr, piece)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{PieceColor, WHITE_ROOK},
    };

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
        let board = boards.boards[bitboard_idx(WHITE_ROOK)];

        let expected = Bitboards::from_str(
            r#"
            00r00
            00r00
            rr0rr
            00r00
            00r00
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_ROOK)];
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
        let board = boards.boards[bitboard_idx(WHITE_ROOK)];

        let expected = Bitboards::from_str(
            r#"
            00r00
            00r00
            rr0rr
            00r00
            00r00
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_ROOK)];
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
        let board = boards.boards[bitboard_idx(WHITE_ROOK)];

        let mut plys: BinaryHeap<Ply> = board
            .rook_plys(
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.boards.as_ptr(),
                WHITE_ROOK,
            )
            .collect();
        assert_eq!(plys.len(), 8);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
