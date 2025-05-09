use super::bitboard::Bitboard;
use bevy::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const PIECE_TYPE_COUNT: usize = 6;
pub const PIECE_COLOR_COUNT: usize = 2;
pub const PIECE_COMBO_COUNT: usize = PIECE_TYPE_COUNT * PIECE_COLOR_COUNT;

pub const WHITE_KING: Piece = Piece(PieceType::King, PieceColor::White);
pub const BLACK_KING: Piece = Piece(PieceType::King, PieceColor::Black);
pub const WHITE_QUEEN: Piece = Piece(PieceType::Queen, PieceColor::White);
pub const BLACK_QUEEN: Piece = Piece(PieceType::Queen, PieceColor::Black);
pub const WHITE_ROOK: Piece = Piece(PieceType::Rook, PieceColor::White);
pub const BLACK_ROOK: Piece = Piece(PieceType::Rook, PieceColor::Black);
pub const WHITE_BISHOP: Piece = Piece(PieceType::Bishop, PieceColor::White);
pub const BLACK_BISHOP: Piece = Piece(PieceType::Bishop, PieceColor::Black);
pub const WHITE_KNIGHT: Piece = Piece(PieceType::Knight, PieceColor::White);
pub const BLACK_KNIGHT: Piece = Piece(PieceType::Knight, PieceColor::Black);
pub const WHITE_PAWN: Piece = Piece(PieceType::Pawn, PieceColor::White);
pub const BLACK_PAWN: Piece = Piece(PieceType::Pawn, PieceColor::Black);

// /// To be removed
// #[derive(Component, Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
// pub struct LegacyPiece {
//     pub piece_type: PieceType,
//     pub color: PieceColor,
//     pub pos: Pos,
//     pub has_moved: bool,
// }

#[derive(Component, Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Piece(pub PieceType, pub PieceColor);
impl Piece {
    pub fn as_char(&self) -> char {
        match *self {
            WHITE_KING => 'k',
            BLACK_KING => 'K',
            WHITE_QUEEN => 'q',
            BLACK_QUEEN => 'Q',
            WHITE_ROOK => 'r',
            BLACK_ROOK => 'R',
            WHITE_BISHOP => 'b',
            BLACK_BISHOP => 'B',
            WHITE_KNIGHT => 'n',
            BLACK_KNIGHT => 'N',
            WHITE_PAWN => 'p',
            BLACK_PAWN => 'P',
        }
    }

    /// Full iter through all possible PieceType, PieceColor combinations
    pub fn iter() -> impl Iterator<Item = Self> {
        PieceType::iter()
            .flat_map(|piece_type| PieceColor::iter().map(move |color| Piece(piece_type, color)))
    }

    /// Iter through all possible PieceType of a particular color
    pub fn iter_color(color: PieceColor) -> impl Iterator<Item = Self> + Clone {
        PieceType::iter().map(move |piece_type| Piece(piece_type, color))
    }
}

impl From<char> for Piece {
    fn from(value: char) -> Self {
        match value {
            'k' => WHITE_KING,
            'K' => BLACK_KING,
            'q' => WHITE_QUEEN,
            'Q' => BLACK_QUEEN,
            'r' => WHITE_ROOK,
            'R' => BLACK_ROOK,
            'b' => WHITE_BISHOP,
            'B' => BLACK_BISHOP,
            'n' => WHITE_KNIGHT,
            'N' => BLACK_KNIGHT,
            'p' => WHITE_PAWN,
            'P' => BLACK_PAWN,
            _ => panic!("Unexpected char: {}", value),
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PieceWithBitboard(pub Piece, pub Bitboard);

// impl LegacyPiece {
//     pub fn new(piece_type: PieceType, color: PieceColor, pos: Pos) -> Self {
//         Self {
//             piece_type,
//             color,
//             pos,
//             ..Default::default()
//         }
//     }

//     pub fn to_char(&self) -> char {
//         match Piece(self.piece_type, self.color) {
//             WHITE_KING => 'k',
//             BLACK_KING => 'K',
//             WHITE_QUEEN => 'q',
//             BLACK_QUEEN => 'Q',
//             WHITE_ROOK => 'r',
//             BLACK_ROOK => 'R',
//             WHITE_BISHOP => 'b',
//             BLACK_BISHOP => 'B',
//             WHITE_KNIGHT => 'n',
//             BLACK_KNIGHT => 'N',
//             WHITE_PAWN => 'p',
//             BLACK_PAWN => 'P',
//         }
//     }

//     /// Used with MVV-LVA for move ordering.
//     /// Uses the discriminant from the enum to compare
//     pub fn attacker_cmp(&self, other: &Self) -> Ordering {
//         self.piece_type
//             .discriminant()
//             .cmp(&other.piece_type.discriminant())
//     }

//     /// The generated moves do not perform any checking checks, however vector attacks do stop at collisions
//     pub fn generate_pseudolegal_moves(&self, board: &Game) -> Vec<LegacyPly> {
//         match self.piece_type {
//             PieceType::King => self.king_move_generation(board),
//             PieceType::Queen => self.queen_move_generation(board),
//             PieceType::Rook => self.rook_move_generation(board),
//             PieceType::Bishop => self.bishop_move_generation(board),
//             PieceType::Knight => self.knight_move_generation(board),
//             PieceType::Pawn => self.pawn_move_generation(board),
//         }
//     }

//     fn king_move_generation(&self, board: &Game) -> Vec<LegacyPly> {
//         let movement_vectors = [
//             MoveVec { x: 1, y: 0 },
//             MoveVec { x: 1, y: 1 },
//             MoveVec { x: 0, y: 1 },
//             MoveVec { x: -1, y: 1 },
//             MoveVec { x: -1, y: 0 },
//             MoveVec { x: -1, y: -1 },
//             MoveVec { x: 0, y: -1 },
//             MoveVec { x: 1, y: -1 },
//         ];

//         self.step_moves(board, &movement_vectors)
//     }

//     fn knight_move_generation(&self, board: &Game) -> Vec<LegacyPly> {
//         let movement_vectors = [
//             MoveVec { x: 1, y: 2 },
//             MoveVec { x: 2, y: 1 },
//             MoveVec { x: -1, y: 2 },
//             MoveVec { x: 2, y: -1 },
//             MoveVec { x: -2, y: 1 },
//             MoveVec { x: 1, y: -2 },
//             MoveVec { x: -1, y: -2 },
//             MoveVec { x: -2, y: -1 },
//         ];

//         self.step_moves(board, &movement_vectors)
//     }

//     fn step_moves(&self, board: &Game, movement_vectors: &[MoveVec]) -> Vec<LegacyPly> {
//         let mut moves: Vec<LegacyPly> = vec![];
//         for vec in movement_vectors {
//             // `valid_dest` is definitely in range of the board
//             if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, vec) {
//                 // perform `Tile::Inactive` and destination piece color checks
//                 let tile = board[valid_dest];
//                 match tile {
//                     None => moves.push(self.move_to_pos(valid_dest)),
//                     Some(other_piece) if self.color != other_piece.color => {
//                         moves.push(self.move_capture(&other_piece))
//                     }
//                     _ => (),
//                 }
//             }
//         }

//         moves
//     }

//     fn raycasted_moves(&self, board: &Game, movement_vectors: &[MoveVec]) -> Vec<LegacyPly> {
//         let mut moves: Vec<LegacyPly> = vec![];
//         for vec in movement_vectors {
//             self.vector_walk(board, &mut moves, vec);
//         }

//         moves
//     }

//     fn rook_move_generation(&self, board: &Game) -> Vec<LegacyPly> {
//         let movement_vectors = [
//             MoveVec { x: 1, y: 0 },
//             MoveVec { x: 0, y: 1 },
//             MoveVec { x: -1, y: 0 },
//             MoveVec { x: 0, y: -1 },
//         ];

//         self.raycasted_moves(board, &movement_vectors)
//     }

//     fn bishop_move_generation(&self, board: &Game) -> Vec<LegacyPly> {
//         let movement_vectors = [
//             MoveVec { x: 1, y: 1 },
//             MoveVec { x: -1, y: 1 },
//             MoveVec { x: -1, y: -1 },
//             MoveVec { x: 1, y: -1 },
//         ];
//         self.raycasted_moves(board, &movement_vectors)
//     }

//     fn queen_move_generation(&self, board: &Game) -> Vec<LegacyPly> {
//         let movement_vectors = [
//             MoveVec { x: 1, y: 0 },
//             MoveVec { x: 1, y: 1 },
//             MoveVec { x: 0, y: 1 },
//             MoveVec { x: -1, y: 1 },
//             MoveVec { x: -1, y: 0 },
//             MoveVec { x: -1, y: -1 },
//             MoveVec { x: 0, y: -1 },
//             MoveVec { x: 1, y: -1 },
//         ];

//         self.raycasted_moves(board, &movement_vectors)
//     }

//     fn pawn_move_generation(&self, board: &Game) -> Vec<LegacyPly> {
//         let mut moves: Vec<LegacyPly> = vec![];

//         let direction = if self.color == PieceColor::White {
//             -1
//         } else {
//             1
//         };
//         let move_vec = MoveVec {
//             x: 1 * direction,
//             y: 0,
//         };

//         // forward move
//         if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &move_vec) {
//             // Checking that the pawn isn't blocked
//             if board[valid_dest].is_none() {
//                 moves.push(self.move_to_pos(valid_dest));

//                 // If the first step isn't blocked, we can check for the double step
//                 if self.piece_type == PieceType::Pawn && !self.has_moved {
//                     let move_vec = MoveVec {
//                         x: 2 * direction,
//                         y: 0,
//                     };
//                     if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &move_vec) {
//                         // Checking that the pawn isn't blocked
//                         if board[valid_dest].is_none() {
//                             moves.push(self.move_to_pos_en_passant(valid_dest));
//                         }
//                     }
//                 }
//             }
//         }

//         // diagonal taking
//         let move_vecs = [
//             MoveVec {
//                 x: 1 * direction,
//                 y: -1,
//             },
//             MoveVec {
//                 x: 1 * direction,
//                 y: 1,
//             },
//         ];
//         for vec in move_vecs {
//             if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &vec) {
//                 let other_piece = board[valid_dest];
//                 if let Some(other_piece) = other_piece {
//                     if other_piece.color != self.color {
//                         moves.push(self.move_capture(&other_piece));
//                     }
//                 }
//             }
//         }

//         // en passant
//         let last_move = board.moves.last().cloned();

//         if let Some(last_move) = last_move {
//             if last_move.en_passant_flag
//                 && last_move.move_to.to.row == self.pos.row
//                 && last_move.move_to.to.column.abs_diff(self.pos.column) == 1
//             {
//                 let move_vec = MoveVec {
//                     x: direction,
//                     y: last_move.move_to.to.column as i16 - self.pos.column as i16,
//                 };
//                 if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &move_vec) {
//                     if let Some(other_piece) = board[last_move.move_to.to] {
//                         if other_piece.color != self.color {
//                             moves.push(self.move_to_while_capturing(valid_dest, &other_piece));
//                         }
//                     }
//                 }
//             }
//         }

//         moves
//     }
//     /// walk into the direction of a MoveVec until we reach and `Tile::Inactive`, a piece of our color,
//     /// or we pass a piece of the opponent's color.
//     fn vector_walk(&self, board: &Game, moves: &mut Vec<LegacyPly>, vec: &MoveVec) {
//         let mut finished = false;
//         let mut pos = self.pos;
//         while !finished {
//             if let Some(valid_dest) = board.apply_vec_to_pos(pos, vec) {
//                 // perform out of bound and destination piece color checks
//                 let tile = board[valid_dest];
//                 match tile {
//                     None => {
//                         moves.push(self.move_to_pos(valid_dest));
//                         pos = valid_dest
//                     }
//                     Some(other_piece) if self.color != other_piece.color => {
//                         moves.push(self.move_capture(&other_piece));
//                         finished = true;
//                     }
//                     _ => finished = true,
//                 }
//             } else {
//                 finished = true;
//             }
//         }
//     }

//     fn move_to_pos(&self, pos: Pos) -> LegacyPly {
//         LegacyPly {
//             move_to: self.pos.move_to(&pos),
//             by: *self,
//             ..Default::default()
//         }
//     }

//     fn move_to_pos_en_passant(&self, pos: Pos) -> LegacyPly {
//         LegacyPly {
//             move_to: self.pos.move_to(&pos),
//             by: *self,
//             en_passant_flag: true,
//             ..Default::default()
//         }
//     }

//     fn move_capture(&self, piece: &LegacyPiece) -> LegacyPly {
//         LegacyPly {
//             move_to: self.pos.move_to(&piece.pos),
//             by: *self,
//             capturing: Some(*piece),
//             ..Default::default()
//         }
//     }

//     fn move_to_while_capturing(&self, pos: Pos, piece: &LegacyPiece) -> LegacyPly {
//         LegacyPly {
//             move_to: self.pos.move_to(&pos),
//             by: *self,
//             capturing: Some(*piece),
//             ..Default::default()
//         }
//     }
// }

#[derive(
    Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter, PartialOrd, Ord,
)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    #[default]
    Pawn,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum PieceColor {
    #[default]
    White,
    Black,
}
impl PieceColor {
    pub fn score_sign(&self) -> i32 {
        match self {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn king_move_generation() {
    //     let game = Game::new_from_str(
    //         r#"
    //     0kP
    //     p00
    //     "#,
    //         3,
    //     );
    //     let pseudolegal_count = 4;

    //     let piece = game[Pos::new(0, 1)].unwrap();
    //     let enemy_pawn = game[Pos::new(0, 2)].unwrap();
    //     let friend_pawn = game[Pos::new(1, 0)].unwrap();

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    // }

    // #[test]
    // fn rook_move_generation() {
    //     let game = Game::new_from_str(
    //         r#"
    //     prPP
    //     0000
    //     0000
    //     "#,
    //         4,
    //     );
    //     let pseudolegal_count = 3;

    //     let piece = game[Pos::new(0, 1)].unwrap();
    //     let enemy_pawn = game[Pos::new(0, 2)].unwrap();
    //     let obscured_pawn = game[Pos::new(0, 3)].unwrap();
    //     let friend_pawn = game[Pos::new(0, 0)].unwrap();
    //     let far_away_pos = Pos::new(2, 1);

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&obscured_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    //     assert!(moves.contains(&piece.move_to_pos(far_away_pos)));
    // }

    // #[test]
    // fn bishop_move_generation() {
    //     let game = Game::new_from_str(
    //         r#"
    //     0000P
    //     0p0P0
    //     00b00
    //     00000
    //     00000
    //     "#,
    //         5,
    //     );
    //     let pseudolegal_count = 5;

    //     let piece = game[Pos::new(2, 2)].unwrap();
    //     let enemy_pawn = game[Pos::new(1, 3)].unwrap();
    //     let obscured_pawn = game[Pos::new(0, 4)].unwrap();
    //     let friend_pawn = game[Pos::new(1, 1)].unwrap();
    //     let far_away_pos = Pos::new(4, 0);

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&obscured_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    //     assert!(moves.contains(&piece.move_to_pos(far_away_pos)));
    // }

    // #[test]
    // fn queen_move_generation() {
    //     let game = Game::new_from_str(
    //         r#"
    //     0000P
    //     0p0P0
    //     00q00
    //     00000
    //     00RN0
    //     "#,
    //         5,
    //     );
    //     let pseudolegal_count = 13;

    //     let piece = game[Pos::new(2, 2)].unwrap();
    //     let enemy_pawn = game[Pos::new(1, 3)].unwrap();
    //     let obscured_pawn = game[Pos::new(0, 4)].unwrap();
    //     let friend_pawn = game[Pos::new(1, 1)].unwrap();
    //     let far_away_rook = game[Pos::new(4, 2)].unwrap();
    //     let untargeted_knight = game[Pos::new(4, 3)].unwrap();

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&obscured_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    //     assert!(moves.contains(&piece.move_capture(&far_away_rook)));
    //     assert!(!moves.contains(&piece.move_capture(&untargeted_knight)));
    // }

    // #[test]
    // fn knight_move_generation() {
    //     let game = Game::new_from_str(
    //         r#"
    //     000P0
    //     0000p
    //     00n00
    //     00000
    //     00000
    //     "#,
    //         5,
    //     );
    //     let pseudolegal_count = 7;

    //     let piece = game[Pos::new(2, 2)].unwrap();
    //     let enemy_pawn = game[Pos::new(0, 3)].unwrap();
    //     let friend_pawn = game[Pos::new(1, 4)].unwrap();

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
    //     assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    // }

    // #[test]
    // fn pawn_move_generation() {
    //     let game = Game::new_from_str(
    //         r#"
    //     0000
    //     00P0
    //     0p00
    //     "#,
    //         4,
    //     );
    //     let pseudolegal_count = 3;

    //     let piece = game[Pos::new(2, 1)].unwrap();
    //     let enemy_pawn = game[Pos::new(1, 2)].unwrap();
    //     let normal_pos = Pos::new(1, 1);
    //     let double_step_pos = Pos::new(0, 1);

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), pseudolegal_count);
    //     assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
    //     assert!(moves.contains(&piece.move_to_pos(normal_pos)));
    //     assert!(moves.contains(&piece.move_to_pos_en_passant(double_step_pos)));
    // }

    // #[test]
    // fn keep_track_of_kings_test() {
    //     let game = Game::new_from_str(
    //         r#"
    //         0k
    //         K0
    //         "#,
    //         2,
    //     );

    //     assert_eq!(
    //         game.piece_map.get(&(PieceColor::White, PieceType::King)),
    //         Some(&vec![Pos::new(0, 1)])
    //     );
    //     assert_eq!(
    //         game.piece_map.get(&(PieceColor::Black, PieceType::King)),
    //         Some(&vec![Pos::new(1, 0)])
    //     );
    // }

    // #[test]
    // fn pawn_en_passant_test() {
    //     let mut game = Game::new_from_str(
    //         r#"
    //         00
    //         00
    //         Pp
    //         "#,
    //         2,
    //     );
    //     game.moves.push(super::LegacyPly {
    //         move_to: MoveTo {
    //             from: Pos::new(0, 0),
    //             to: Pos::new(2, 0),
    //         },
    //         by: LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::Black,
    //             pos: Pos::new(0, 0),
    //             has_moved: false,
    //         },
    //         en_passant_flag: true,
    //         ..Default::default()
    //     });
    //     let valid_moves = 3;

    //     let piece = game[Pos::new(2, 1)].unwrap();
    //     let enemy_pawn = game[Pos::new(2, 0)].unwrap();
    //     let dest = Pos::new(1, 0);

    //     let moves = piece.generate_pseudolegal_moves(&game);
    //     assert_eq!(moves.len(), valid_moves);
    //     assert!(moves.contains(&piece.move_to_while_capturing(dest, &enemy_pawn)));
    // }

    // #[test]
    // fn legal_moves_checking_test() {
    //     let game = Game::new_from_str(
    //         r#"
    //         00k0n
    //         r00b0
    //         00R00
    //         "#,
    //         5,
    //     );
    //     let legal_moves = 6;

    //     let moves = game.sorted_move_list();

    //     assert_eq!(moves.len(), legal_moves);
    // }

    // #[test]
    // fn kingless_legal_move_test() {
    //     let board = Game::new_from_str(
    //         r#"
    //         0000n
    //         r00b0
    //         00R00
    //         "#,
    //         5,
    //     );
    //     let legal_moves = 9;

    //     let moves = board.sorted_move_list();

    //     assert_eq!(moves.len(), legal_moves);
    // }
}
