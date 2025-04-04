use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    pub fn king_move_mask(&self) -> Self {
        self.king_move_arr()
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap()
    }

    pub fn king_move_arr(&self) -> [Self; 8] {
        [
            self.shift_we(),
            self.shift_nw(),
            self.shift_no(),
            self.shift_ne(),
            self.shift_ea(),
            self.shift_se(),
            self.shift_so(),
            self.shift_sw(),
        ]
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
        let mask = board.king_move_mask();
        assert_eq!(mask, expected);
    }
}
