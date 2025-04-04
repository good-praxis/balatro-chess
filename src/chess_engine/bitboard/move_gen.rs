use super::Bitboard;

pub mod king;
pub mod knight;

impl Bitboard {
    // Common single-step shifts
    // King-like one-steps
    fn shift_we(&self) -> Self {
        *self >> 1
    }

    fn shift_nw(&self) -> Self {
        *self >> 17
    }

    fn shift_no(&self) -> Self {
        *self >> 16
    }

    fn shift_ne(&self) -> Self {
        *self >> 15
    }

    fn shift_ea(&self) -> Self {
        *self << 1
    }

    fn shift_se(&self) -> Self {
        *self << 17
    }

    fn shift_so(&self) -> Self {
        *self << 16
    }

    fn shift_sw(&self) -> Self {
        *self << 15
    }

    // Knight-like one-steps
    fn shift_nww(&self) -> Self {
        *self >> (16 + 1 + 1)
    }

    fn shift_nnw(&self) -> Self {
        *self >> (16 + 16 + 1)
    }

    fn shift_nne(&self) -> Self {
        *self >> (16 + 16 - 1)
    }

    fn shift_nee(&self) -> Self {
        *self >> (16 - 1 - 1)
    }

    fn shift_see(&self) -> Self {
        *self << (16 + 1 + 1)
    }

    fn shift_sse(&self) -> Self {
        *self << (16 + 16 + 1)
    }

    fn shift_ssw(&self) -> Self {
        *self << (16 + 16 - 1)
    }

    fn shift_sww(&self) -> Self {
        *self << (16 - 1 - 1)
    }

    // limit via `BitAnd` of the `limit` board
    fn limit(&self, limit: &Self) -> Self {
        *self & *limit
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
        pieces::{PieceColor, PieceType},
    };

    use super::*;

    fn king_board() -> Bitboard {
        let boards = Bitboards::from_str(
            r#"
            000
            0k0
            000
            "#,
        );
        boards.boards[bitboard_idx(PieceType::King, PieceColor::White)]
    }

    #[test]
    fn shift_west() {
        let board = king_board();
        let shift = board.shift_we();
        assert!(board.get(17));
        assert!(shift.get(16));
    }

    #[test]
    fn shift_northweast() {
        let board = king_board();
        let shift = board.shift_nw();
        assert!(board.get(17));
        assert!(shift.get(0));
    }

    #[test]
    fn shift_north() {
        let board = king_board();
        let shift = board.shift_no();
        assert!(board.get(17));
        assert!(shift.get(1));
    }

    #[test]
    fn shift_northeast() {
        let board = king_board();
        let shift = board.shift_ne();
        assert!(board.get(17));
        assert!(shift.get(2));
    }

    #[test]
    fn shift_east() {
        let board = king_board();
        let shift = board.shift_ea();
        assert!(board.get(17));
        assert!(shift.get(18));
    }

    #[test]
    fn shift_southeast() {
        let board = king_board();
        let shift = board.shift_se();
        assert!(board.get(17));
        assert!(shift.get(34));
    }

    #[test]
    fn shift_south() {
        let board = king_board();
        let shift = board.shift_so();
        assert!(board.get(17));
        assert!(shift.get(33));
    }

    #[test]
    fn shift_southwest() {
        let board = king_board();
        let shift = board.shift_sw();
        assert!(board.get(17));
        assert!(shift.get(32));
    }

    fn knight_board() -> Bitboard {
        let boards = Bitboards::from_str(
            r#"
            00000
            00000
            00n00
            00000
            00000
            "#,
        );
        boards.boards[bitboard_idx(PieceType::Knight, PieceColor::White)]
    }

    #[test]
    fn shift_northwestwest() {
        let board = knight_board();
        let shift = board.shift_nww();
        assert!(board.get(34));
        assert!(shift.get(16));
    }

    #[test]
    fn shift_northnorthwest() {
        let board = knight_board();
        let shift = board.shift_nnw();
        assert!(board.get(34));
        assert!(shift.get(1));
    }

    #[test]
    fn shift_northnortheast() {
        let board = knight_board();
        let shift = board.shift_nne();
        assert!(board.get(34));
        assert!(shift.get(3));
    }

    #[test]
    fn shift_northeasteast() {
        let board = knight_board();
        let shift = board.shift_nee();
        assert!(board.get(34));
        assert!(shift.get(20));
    }

    #[test]
    fn shift_southeasteast() {
        let board = knight_board();
        let shift = board.shift_see();
        assert!(board.get(34));
        assert!(shift.get(52));
    }

    #[test]
    fn shift_southsoutheast() {
        let board = knight_board();
        let shift = board.shift_sse();
        assert!(board.get(34));
        assert!(shift.get(67));
    }

    #[test]
    fn shift_southsouthwest() {
        let board = knight_board();
        let shift = board.shift_ssw();
        assert!(board.get(34));
        assert!(shift.get(65));
    }

    #[test]
    fn shift_southwestwest() {
        let board = knight_board();
        let shift = board.shift_sww();
        assert!(board.get(34));
        assert!(shift.get(48));
    }

    #[test]
    fn limit_out_of_range() {
        let boards = Bitboards::from_str(
            r#"
            000
            000
            00k
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];
        let res = board.king_move_mask().limit(&boards.limits);
        assert_eq!(res.count_ones(), 3);

        let boards = Bitboards::from_str(
            r#"
            k00
            000
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];
        let res = board.king_move_mask().limit(&boards.limits);
        assert_eq!(res.count_ones(), 3);
    }
}
