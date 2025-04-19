use ethnum::u256;

use crate::chess_engine::{bitboard::Bitboard, pieces::Piece};

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
        self.king_moves(blocked, _capturable)
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap_or(Bitboard(u256::ZERO))
    }

    /// Pseudolegal moves by king
    pub fn king_moves(&self, blocked: &Self, _capturable: &Self) -> impl Iterator<Item = Bitboard> {
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
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        self.single_step_plys_in_dirs(&KING_DIRS, blocked, capturable, bitboard_ptr, piece)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{PieceColor, WHITE_KING},
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
        let board = boards.boards[bitboard_idx(WHITE_KING)];

        let expected = Bitboards::from_str(
            r#"
            kk0
            k0k
            kkk
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_KING)];
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
        let board = boards.boards[bitboard_idx(WHITE_KING)];

        let expected = Bitboards::from_str(
            r#"
            0k
            kk
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_KING)];
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
        let board = boards.boards[bitboard_idx(WHITE_KING)];

        let vec: Vec<_> = board
            .king_moves(
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
            )
            .collect();
        assert_eq!(vec.len(), 3);
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
        let board = boards.boards[bitboard_idx(WHITE_KING)];

        let expected = Bitboards::from_str(
            r#"
            kkk
            k0k
            kkk
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_KING)];
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
        let board = boards.boards[bitboard_idx(WHITE_KING)];

        let mut plys: BinaryHeap<Ply> = board
            .king_plys(
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.boards.as_ptr(),
                WHITE_KING,
            )
            .collect();
        assert_eq!(plys.len(), 8);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
