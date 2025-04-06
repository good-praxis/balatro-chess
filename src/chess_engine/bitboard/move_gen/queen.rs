use crate::chess_engine::bitboard::Bitboard;

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
        let dirs = [
            Self::step_we,
            Self::step_nw,
            Self::step_no,
            Self::step_ne,
            Self::step_ea,
            Self::step_se,
            Self::step_so,
            Self::step_sw,
        ];
        self.step_in_dirs(&dirs, blocked, capturable)
    }

    /// Mask of threatened positions
    pub fn queen_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.queen_move_mask(blocked, capturable)
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
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
}
