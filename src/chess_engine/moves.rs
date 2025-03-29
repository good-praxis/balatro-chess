use super::pieces::Piece;

/// Valid position on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub row: usize,
    pub column: usize,
}
impl Pos {
    pub fn new(row: usize, column: usize) -> Self {
        Pos { row, column }
    }

    pub fn move_to(&self, other: &Pos) -> MoveTo {
        MoveTo {
            from: *self,
            to: *other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveTo {
    pub from: Pos,
    pub to: Pos,
}

/// Used with a Pos to generate a potentially new valid Pos on the board
#[derive(Debug, Clone, Copy)]
pub struct MoveVec {
    pub x: i16,
    pub y: i16,
}

/// Type that encodes the `MoveTo` with metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub move_to: MoveTo,
    pub by: Piece,
    /// Set if a move takes an opponent's piece
    pub capturing: Option<Piece>,
    /// Set if this move allows a pawn to be attacked via en passant
    pub en_passant_flag: bool,
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess_engine::board::Board;

    #[test]
    fn apply_vec_to_pos() {
        let board = Board::from_str(
            r#"
            000
            000
            000
            000
            "#,
            3,
        );

        let pos = Pos::new(1, 1);
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: 1, y: 1 })
                .is_some()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: -1, y: -1 })
                .is_some()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: 2, y: 1 })
                .is_some()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: 3, y: 1 })
                .is_none()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: -2, y: 1 })
                .is_none()
        );
    }
}
