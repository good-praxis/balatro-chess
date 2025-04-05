use crate::chess_engine::bitboard::Bitboard;

impl Bitboard {
    /// Cumulative pseudolegal unlimited mask of knight moves
    pub fn knight_move_mask(&self) -> Self {
        self.knight_move_arr()
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap()
    }

    /// Pseudolegal unlimited moves by knight
    pub fn knight_move_arr(&self) -> [Self; 8] {
        [
            self.shift_nww(),
            self.shift_nnw(),
            self.shift_nne(),
            self.shift_nee(),
            self.shift_see(),
            self.shift_sse(),
            self.shift_ssw(),
            self.shift_sww(),
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
    fn knight_move_mask() {
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
        let mask = board.knight_move_mask();
        assert_eq!(mask, expected);
    }
}
