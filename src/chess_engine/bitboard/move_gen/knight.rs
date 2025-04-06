use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal mask of knight moves
    pub fn knight_move_mask(&self, blocked: &Self, _capturable: &Self) -> Self {
        self.knight_move_arr(blocked, _capturable)
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap()
    }

    /// Pseudolegal moves by knight
    pub fn knight_move_arr(&self, blocked: &Self, _capturable: &Self) -> Vec<Self> {
        let dirs = [
            Self::shift_nww,
            Self::shift_nnw,
            Self::shift_nne,
            Self::shift_nee,
            Self::shift_see,
            Self::shift_sse,
            Self::shift_ssw,
            Self::shift_sww,
        ];

        self.shift_in_dirs(&dirs, blocked, _capturable)
    }

    /// Mask of threatened positions
    pub fn knight_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.knight_move_mask(blocked, capturable)
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
        pieces::{PieceColor, PieceType},
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
        let board = boards.boards[bitboard_idx(PieceType::Knight, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            0n0n0
            n0000
            00000
            n000n
            0n0n0
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Knight, PieceColor::White)];
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
        let board = boards.boards[bitboard_idx(PieceType::Knight, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            0n0n0
            n000n
            00000
            n000n
            0n0n0
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Knight, PieceColor::White)];
        let mask = board.knight_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
        );
        assert_eq!(mask, expected);
    }
}
