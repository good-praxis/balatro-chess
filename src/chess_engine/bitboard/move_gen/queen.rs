use crate::chess_engine::{bitboard::Bitboard, pieces::Piece};

use super::ply::Ply;

pub(crate) const QUEEN_STEP_DIRS: [fn(&Bitboard) -> Bitboard; 8] = [
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
    /// Cumulative pseudolegal mask of queen moves
    pub fn queen_move_mask(&self, blocked: &Bitboard, capturable: &Bitboard) -> Self {
        let dirs = [
            Self::fill_we,
            Self::fill_nw,
            Self::fill_no,
            Self::fill_ne,
            Self::fill_ea,
            Self::fill_se,
            Self::fill_so,
            Self::fill_sw,
        ];

        self.fill_in_dirs(&dirs, blocked, capturable)
    }

    /// Mask of threatened positions
    pub fn queen_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.queen_move_mask(blocked, capturable)
    }

    pub fn queen_plys(
        &self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        self.multi_step_plys_in_dirs(&QUEEN_STEP_DIRS, blocked, capturable, bitboard_ptr, piece)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{PieceColor, WHITE_QUEEN},
    };

    #[test]
    fn queen_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            0000P
            00000
            p0q00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_QUEEN)];

        let expected = Bitboards::from_str(
            r#"
            q0q0q
            0qqq0
            0q0qq
            0qqq0
            q0q0q
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_QUEEN)];
        let mask = board.queen_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn queen_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            0000P
            00000
            00q00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_QUEEN)];

        let expected = Bitboards::from_str(
            r#"
            q0q0q
            0qqq0
            qq0qq
            0qqq0
            q0q0q
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_QUEEN)];
        let mask = board.queen_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn queen_plys() {
        let boards = Bitboards::from_str(
            r#"
            0000P
            00000
            00q00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_QUEEN)];

        let mut plys: BinaryHeap<Ply> = board
            .queen_plys(
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.boards.as_ptr(),
                WHITE_QUEEN,
            )
            .collect();
        assert_eq!(plys.len(), 16);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
