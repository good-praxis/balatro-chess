use super::Bitboard;

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod ply;
pub mod queen;
pub mod rook;

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

    /// Returns a vector of all unblocked shifted positions for single-step movement pieces
    fn shift_in_dirs(
        &self,
        dirs: &[fn(&Self) -> Self],
        blocked: &Self,
        _capturable: &Self,
    ) -> Vec<Self> {
        dirs.iter()
            .map(|dir| dir(self))
            .filter(|board| **board != 0 && **board & **blocked == 0)
            .collect()
    }

    // fill-in-direction until running into a `blocked` bit (exclusive) or `capturable` bit (inclusive)
    fn fill_dir(&self, dir: fn(&Self) -> Self, blocked: &Self, capturable: &Self) -> Self {
        let mut board = Bitboard(0);
        let mut current = dir(self);
        while *current != 0 && *current & **blocked == 0 {
            board |= current;
            if *current & **capturable != 0 {
                break;
            }
            current = dir(&current);
        }
        board
    }

    /// Returns a bitmask filled into all directions until running into a blocked (exclusive) or capturable (inclusive) bit
    pub fn fill_in_dirs(
        &self,
        dirs: &[fn(&Self, &Self, &Self) -> Self],
        blocked: &Self,
        capturable: &Self,
    ) -> Self {
        dirs.iter()
            .map(|dir| dir(self, blocked, capturable))
            .reduce(|acc, e| acc | e)
            .unwrap_or(Self(0))
    }

    fn fill_we(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_we, blocked, capturable)
    }

    fn fill_nw(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_nw, blocked, capturable)
    }

    fn fill_no(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_no, blocked, capturable)
    }

    fn fill_ne(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_ne, blocked, capturable)
    }
    fn fill_ea(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_ea, blocked, capturable)
    }
    fn fill_se(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_se, blocked, capturable)
    }

    fn fill_so(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_so, blocked, capturable)
    }
    fn fill_sw(&self, blocked: &Self, capturable: &Self) -> Self {
        self.fill_dir(Bitboard::shift_sw, blocked, capturable)
    }

    // step-in-direction until running into a `blocked` bit (exclusive) or `capturable` bit (inclusive). Returns a Vec of Bitboards
    fn step_dir(&self, dir: fn(&Self) -> Self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        let mut steps = vec![];
        let mut current = dir(self);
        while *current != 0 && *current & **blocked == 0 {
            steps.push(current);
            if *current & **capturable != 0 {
                break;
            }
            current = dir(&current);
        }
        steps
    }

    /// Steps towards directions until running into a blocked (exclusive) or capturable (inclusive) bit
    fn step_in_dirs(
        &self,
        dirs: &[fn(&Self, &Self, &Self) -> Vec<Self>],
        blocked: &Self,
        capturable: &Self,
    ) -> Vec<Self> {
        dirs.iter()
            .map(|dir| dir(self, blocked, capturable))
            .flatten()
            .collect()
    }

    fn step_we(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_we, blocked, capturable)
    }

    fn step_nw(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_nw, blocked, capturable)
    }

    fn step_no(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_no, blocked, capturable)
    }

    fn step_ne(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_ne, blocked, capturable)
    }
    fn step_ea(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_ea, blocked, capturable)
    }
    fn step_se(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_se, blocked, capturable)
    }

    fn step_so(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_so, blocked, capturable)
    }
    fn step_sw(&self, blocked: &Self, capturable: &Self) -> Vec<Self> {
        self.step_dir(Bitboard::shift_sw, blocked, capturable)
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboards, bitboard_idx},
        pieces::*,
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

        boards.boards[bitboard_idx(WHITE_KING)]
    }

    #[test]
    fn shift_west() {
        let board = king_board();
        let shift = board.shift_we();
        assert!(board.get(&17));
        assert!(shift.get(&16));
    }

    #[test]
    fn shift_northweast() {
        let board = king_board();
        let shift = board.shift_nw();
        assert!(board.get(&17));
        assert!(shift.get(&0));
    }

    #[test]
    fn shift_north() {
        let board = king_board();
        let shift = board.shift_no();
        assert!(board.get(&17));
        assert!(shift.get(&1));
    }

    #[test]
    fn shift_northeast() {
        let board = king_board();
        let shift = board.shift_ne();
        assert!(board.get(&17));
        assert!(shift.get(&2));
    }

    #[test]
    fn shift_east() {
        let board = king_board();
        let shift = board.shift_ea();
        assert!(board.get(&17));
        assert!(shift.get(&18));
    }

    #[test]
    fn shift_southeast() {
        let board = king_board();
        let shift = board.shift_se();
        assert!(board.get(&17));
        assert!(shift.get(&34));
    }

    #[test]
    fn shift_south() {
        let board = king_board();
        let shift = board.shift_so();
        assert!(board.get(&17));
        assert!(shift.get(&33));
    }

    #[test]
    fn shift_southwest() {
        let board = king_board();
        let shift = board.shift_sw();
        assert!(board.get(&17));
        assert!(shift.get(&32));
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
        boards.boards[bitboard_idx(WHITE_KNIGHT)]
    }

    #[test]
    fn shift_northwestwest() {
        let board = knight_board();
        let shift = board.shift_nww();
        assert!(board.get(&34));
        assert!(shift.get(&16));
    }

    #[test]
    fn shift_northnorthwest() {
        let board = knight_board();
        let shift = board.shift_nnw();
        assert!(board.get(&34));
        assert!(shift.get(&1));
    }

    #[test]
    fn shift_northnortheast() {
        let board = knight_board();
        let shift = board.shift_nne();
        assert!(board.get(&34));
        assert!(shift.get(&3));
    }

    #[test]
    fn shift_northeasteast() {
        let board = knight_board();
        let shift = board.shift_nee();
        assert!(board.get(&34));
        assert!(shift.get(&20));
    }

    #[test]
    fn shift_southeasteast() {
        let board = knight_board();
        let shift = board.shift_see();
        assert!(board.get(&34));
        assert!(shift.get(&52));
    }

    #[test]
    fn shift_southsoutheast() {
        let board = knight_board();
        let shift = board.shift_sse();
        assert!(board.get(&34));
        assert!(shift.get(&67));
    }

    #[test]
    fn shift_southsouthwest() {
        let board = knight_board();
        let shift = board.shift_ssw();
        assert!(board.get(&34));
        assert!(shift.get(&65));
    }

    #[test]
    fn shift_southwestwest() {
        let board = knight_board();
        let shift = board.shift_sww();
        assert!(board.get(&34));
        assert!(shift.get(&48));
    }

    #[test]
    fn fill_west() {
        let boards = Bitboards::from_str(
            r#"
            R00r
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            rrr0
            "#,
        )
        .boards[bitboard_idx(WHITE_ROOK)];

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.fill_we(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_east() {
        let boards = Bitboards::from_str(
            r#"
            r00R
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            0rrr
            "#,
        )
        .boards[bitboard_idx(WHITE_ROOK)];

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.fill_ea(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_north() {
        let boards = Bitboards::from_str(
            r#"
            R
            0
            0
            r
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            r
            r
            r
            0
            "#,
        )
        .boards[bitboard_idx(WHITE_ROOK)];

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.fill_no(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_south() {
        let boards = Bitboards::from_str(
            r#"
            r
            0
            0
            R
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            0
            r
            r
            r
            "#,
        )
        .boards[bitboard_idx(WHITE_ROOK)];

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.fill_so(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_northwest() {
        let boards = Bitboards::from_str(
            r#"
            B000
            0000
            0000
            000b
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            b000
            0b00
            00b0
            0000
            "#,
        )
        .boards[bitboard_idx(WHITE_BISHOP)];

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.fill_nw(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_northeast() {
        let boards = Bitboards::from_str(
            r#"
            000B
            0000
            0000
            b000
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            000b
            00b0
            0b00
            0000
            "#,
        )
        .boards[bitboard_idx(WHITE_BISHOP)];

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.fill_ne(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_southeast() {
        let boards = Bitboards::from_str(
            r#"
            b000
            0000
            0000
            000B
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            0000
            0b00
            00b0
            000b
            "#,
        )
        .boards[bitboard_idx(WHITE_BISHOP)];

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.fill_se(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn fill_southwest() {
        let boards = Bitboards::from_str(
            r#"
            000b
            0000
            0000
            B000
            "#,
        );
        let expected = Bitboards::from_str(
            r#"
            0000
            00b0
            0b00
            b000
            "#,
        )
        .boards[bitboard_idx(WHITE_BISHOP)];

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.fill_sw(&!boards.limits, &capturable), expected);
    }

    #[test]
    fn step_west() {
        let boards = Bitboards::from_str(
            r#"
            R00r
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.step_we(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_east() {
        let boards = Bitboards::from_str(
            r#"
            r00R
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.step_ea(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_north() {
        let boards = Bitboards::from_str(
            r#"
            R
            0
            0
            r
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.step_no(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_south() {
        let boards = Bitboards::from_str(
            r#"
            r
            0
            0
            R
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_ROOK)];
        let capturable = boards.boards[bitboard_idx(BLACK_ROOK)];

        assert_eq!(board.step_so(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_northwest() {
        let boards = Bitboards::from_str(
            r#"
            B000
            0000
            0000
            000b
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.step_nw(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_northeast() {
        let boards = Bitboards::from_str(
            r#"
            000B
            0000
            0000
            b000
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.step_ne(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_southeast() {
        let boards = Bitboards::from_str(
            r#"
            b000
            0000
            0000
            000B
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.step_se(&!boards.limits, &capturable).len(), 3);
    }

    #[test]
    fn step_southwest() {
        let boards = Bitboards::from_str(
            r#"
            000b
            0000
            0000
            B000
            "#,
        );

        let board = boards.boards[bitboard_idx(WHITE_BISHOP)];
        let capturable = boards.boards[bitboard_idx(BLACK_BISHOP)];

        assert_eq!(board.step_sw(&!boards.limits, &capturable).len(), 3);
    }
}
