use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal mask of king moves (no castling)
    pub fn king_move_mask(&self, blocked: &Self, _capturable: &Self) -> Self {
        self.king_move_arr(blocked, _capturable)
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap()
    }

    /// Pseudolegal moves by king
    pub fn king_move_arr(&self, blocked: &Self, _capturable: &Self) -> Vec<Self> {
        let dirs = [
            Self::shift_we,
            Self::shift_nw,
            Self::shift_no,
            Self::shift_ne,
            Self::shift_ea,
            Self::shift_se,
            Self::shift_so,
            Self::shift_sw,
        ];
        self.shift_in_dirs(&dirs, blocked, _capturable)
    }

    /// Mask of threatened positions
    pub fn king_en_prise_mask(&self, blocked: &Self, capturable: &Self) -> Self {
        self.king_move_mask(blocked, capturable)
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
}
