use crate::chess_engine::{
    bitboard::Bitboard,
    pieces::{PieceColor, PieceType},
};

use super::ply::Ply;

pub(crate) const QUEEN_STEP_DIRS: [fn(&Bitboard, &Bitboard, &Bitboard) -> Vec<Bitboard>; 8] = [
    Bitboard::step_we,
    Bitboard::step_nw,
    Bitboard::step_no,
    Bitboard::step_ne,
    Bitboard::step_ea,
    Bitboard::step_se,
    Bitboard::step_so,
    Bitboard::step_sw,
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

    /// Pseudolegal moves by queen
    pub fn queen_move_arr(&self, blocked: &Bitboard, capturable: &Bitboard) -> Vec<Self> {
        self.step_in_dirs(&QUEEN_STEP_DIRS, blocked, capturable)
    }

    /// Mask of threatened positions
    pub fn queen_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.queen_move_mask(blocked, capturable)
    }

    pub fn queen_plys_iter(
        &self,
        blocked: &Self,
        capturable: &Self,
        capturable_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        piece: (PieceType, PieceColor),
    ) -> impl Iterator<Item = Ply> {
        self.multi_step_plys_in_dirs(
            &QUEEN_STEP_DIRS,
            blocked,
            capturable,
            capturable_iter,
            piece,
        )
    }

    pub fn queen_plys<T: Default + FromIterator<Ply>>(
        &self,
        blocked: &Self,
        capturable: &Self,
        capturable_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        piece: (PieceType, PieceColor),
    ) -> T {
        self.queen_plys_iter(blocked, capturable, capturable_iter, piece)
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
    fn queen_move_arr() {
        let boards = Bitboards::from_str(
            r#"
            0000P
            00000
            p0q00
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];

        let arr = board.queen_move_arr(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(arr.len(), 15);
    }

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
        let board = boards.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            q0q0q
            0qqq0
            0q0qq
            0qqq0
            q0q0q
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];
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
        let board = boards.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            q0q0q
            0qqq0
            qq0qq
            0qqq0
            q0q0q
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];
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
        let board = boards.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];

        let mut plys: BinaryHeap<Ply> = board.queen_plys(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
            boards.all_piece_types_by_color(PieceColor::Black),
            (PieceType::Queen, PieceColor::White),
        );
        assert_eq!(plys.len(), 16);
        assert!(plys.pop().unwrap().capturing.is_some())
    }
}
