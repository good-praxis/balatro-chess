// use std::cmp::Ordering;

// use super::pieces::LegacyPiece;

// /// Valid position on the board
// #[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
// pub struct Pos {
//     pub row: usize,
//     pub column: usize,
// }
// impl Pos {
//     pub fn new(row: usize, column: usize) -> Self {
//         Pos { row, column }
//     }

//     pub fn move_to(&self, other: &Pos) -> MoveTo {
//         MoveTo {
//             from: *self,
//             to: *other,
//         }
//     }
// }

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// pub struct MoveTo {
//     pub from: Pos,
//     pub to: Pos,
// }

// /// Used with a Pos to generate a potentially new valid Pos on the board
// #[derive(Debug, Clone, Copy)]
// pub struct MoveVec {
//     pub x: i16,
//     pub y: i16,
// }

// /// A classical chess move from either side. Name choosen to avoid rust's protected `move`
// /// contains flags for capturing, castling, promotions
// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// pub struct LegacyPly {
//     pub move_to: MoveTo,
//     pub by: LegacyPiece,
//     /// Set if a move takes an opponent's piece
//     pub capturing: Option<LegacyPiece>,
//     /// Set if this move allows a pawn to be attacked via en passant
//     pub en_passant_flag: bool,
// }

// impl PartialOrd for LegacyPly {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         // using MVV_LVA
//         match (self.capturing, other.capturing) {
//             (None, Some(_)) => return Some(Ordering::Less),
//             (Some(_), None) => return Some(Ordering::Greater),
//             (None, None) => return Some(self.by.attacker_cmp(&other.by)),
//             _ => Some(
//                 self.capture_sorting_value()
//                     .cmp(&other.capture_sorting_value()),
//             ),
//         }
//     }
// }

// impl Ord for LegacyPly {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.partial_cmp(other).unwrap()
//     }
// }

// impl LegacyPly {
//     fn capture_sorting_value(&self) -> u8 {
//         use super::pieces::PieceType;
//         if let Some(captured) = self.capturing {
//             let victim_value = match captured.piece_type {
//                 PieceType::Queen => 25,
//                 PieceType::Rook => 19,
//                 PieceType::Bishop => 13,
//                 PieceType::Knight => 7,
//                 PieceType::Pawn => 1,
//                 _ => 0,
//             };
//             let attacker_value = match self.by.piece_type {
//                 PieceType::Queen => 1,
//                 PieceType::Rook => 2,
//                 PieceType::Bishop => 3,
//                 PieceType::Knight => 4,
//                 PieceType::Pawn => 5,
//                 _ => 0,
//             };
//             return victim_value + attacker_value;
//         } else {
//             0
//         }
//     }
// }

#[cfg(test)]
mod tests {

    // #[test]
    // fn apply_vec_to_pos() {
    //     let game = Game::new_from_str(
    //         r#"
    //         000
    //         000
    //         000
    //         000
    //         "#,
    //         3,
    //     );

    //     let pos = Pos::new(1, 1);
    //     assert!(
    //         game.apply_vec_to_pos(pos, &MoveVec { x: 1, y: 1 })
    //             .is_some()
    //     );
    //     assert!(
    //         game.apply_vec_to_pos(pos, &MoveVec { x: -1, y: -1 })
    //             .is_some()
    //     );
    //     assert!(
    //         game.apply_vec_to_pos(pos, &MoveVec { x: 2, y: 1 })
    //             .is_some()
    //     );
    //     assert!(
    //         game.apply_vec_to_pos(pos, &MoveVec { x: 3, y: 1 })
    //             .is_none()
    //     );
    //     assert!(
    //         game.apply_vec_to_pos(pos, &MoveVec { x: -2, y: 1 })
    //             .is_none()
    //     );
    // }
}
