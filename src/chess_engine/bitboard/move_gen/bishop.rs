use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal unlimited mask of bishop moves
    pub fn bishop_move_mask(&self, blocked: &Bitboard, capturable: &Bitboard) -> Self {
        self.fill_nw(blocked, capturable)
            | self.fill_ne(blocked, capturable)
            | self.fill_se(blocked, capturable)
            | self.fill_sw(blocked, capturable)
    }

    /// Pseudolegal unlimited moves by bishop
    pub fn bishop_move_arr(&self, blocked: &Bitboard, capturable: &Bitboard) -> Vec<Self> {
        let mut moves = self.step_nw(blocked, capturable);
        moves.extend(self.step_ne(blocked, capturable));
        moves.extend(self.step_se(blocked, capturable));
        moves.extend(self.step_sw(blocked, capturable));
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
