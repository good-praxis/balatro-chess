use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal mask of bishop moves
    pub fn bishop_move_mask(&self, blocked: &Bitboard, capturable: &Bitboard) -> Self {
        let dirs = [Self::fill_nw, Self::fill_ne, Self::fill_se, Self::fill_sw];
        self.fill_in_dirs(&dirs, blocked, capturable)
    }

    /// Pseudolegal moves by bishop
    pub fn bishop_move_arr(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        let dirs = [Self::step_nw, Self::step_ne, Self::step_se, Self::step_sw];
        self.step_in_dirs(&dirs, blocked, capturable)
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
        pieces::{PieceColor, PieceType},
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
        let board = boards.boards[bitboard_idx(PieceType::Bishop, PieceColor::White)];

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
        let board = boards.boards[bitboard_idx(PieceType::Bishop, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            0000b
            0b0b0
            00000
            0b0b0
            b000b
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Bishop, PieceColor::White)];
        let mask = board.bishop_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }
}
