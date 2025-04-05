use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal unlimited mask of queen moves
    pub fn queen_move_mask(&self, blocked: &Bitboard, capturable: &Bitboard) -> Self {
        self.fill_we(blocked, capturable)
            | self.fill_nw(blocked, capturable)
            | self.fill_no(blocked, capturable)
            | self.fill_ne(blocked, capturable)
            | self.fill_ea(blocked, capturable)
            | self.fill_se(blocked, capturable)
            | self.fill_so(blocked, capturable)
            | self.fill_sw(blocked, capturable)
    }

    /// Pseudolegal unlimited moves by queen
    pub fn queen_move_arr(&self, blocked: &Bitboard, capturable: &Bitboard) -> Vec<Self> {
        let mut moves = self.step_we(blocked, capturable);
        moves.extend(self.step_nw(blocked, capturable));
        moves.extend(self.step_no(blocked, capturable));
        moves.extend(self.step_ne(blocked, capturable));
        moves.extend(self.step_ea(blocked, capturable));
        moves.extend(self.step_se(blocked, capturable));
        moves.extend(self.step_so(blocked, capturable));
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
}
