use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal unlimited mask of rook moves (no castling)
    pub fn rook_move_mask(&self, blocked: &Bitboard, capturable: &Bitboard) -> Self {
        self.fill_we(blocked, capturable)
            | self.fill_no(blocked, capturable)
            | self.fill_ea(blocked, capturable)
            | self.fill_so(blocked, capturable)
    }

    /// Pseudolegal unlimited moves by rook
    pub fn rook_move_arr(&self, blocked: &Bitboard, capturable: &Bitboard) -> Vec<Self> {
        let mut moves = self.step_we(blocked, capturable);
        moves.extend(self.step_no(blocked, capturable));
        moves.extend(self.step_ea(blocked, capturable));
        moves.extend(self.step_so(blocked, capturable));
        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
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
            00p00
            00000
            00r0P
            00000
            00000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Rook, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            00000
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
}
