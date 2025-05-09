use super::bitboard::Bitboards;
use bevy::prelude::*;
use std::fmt::Display;

#[derive(Resource, Debug, Clone)]
pub struct Game {
    pub boards: Bitboards,
}
impl Default for Game {
    fn default() -> Self {
        Game::new_from_str(
            r#"
        RNBQKBNR
        PPPPPPPP
        00000000
        00000000
        00000000
        00000000
        pppppppp
        rnbqkbnr
        "#,
        )
    }
}
impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board_str = self.boards.to_string();
        write!(f, "{}", board_str)
    }
}

impl Game {
    pub fn new_from_str(input: &str) -> Self {
        Self {
            boards: Bitboards::new_from_str(input),
        }
    }

    // /// Returns the legal moves for a piece at a given position
    // fn get_moves_for_pos(&self, pos: Pos) -> Option<Vec<LegacyPly>> {
    //     let piece = self[pos];

    //     if let Some(piece) = piece {
    //         let moves = piece.generate_pseudolegal_moves(self);
    //         let legal_moves = self.legality_check(moves);

    //         if legal_moves.is_empty() {
    //             None
    //         } else {
    //             Some(legal_moves)
    //         }
    //     } else {
    //         None
    //     }
    // }

    // /// Takes values from 0 to 15 for the intended board-size
    // #[inline]
    // pub fn row_and_column_to_idx(&self, row: usize, column: usize) -> usize {
    //     row * self.row_length + column
    // }

    // #[inline]
    // pub fn pos_to_idx(&self, pos: Pos) -> usize {
    //     self.row_and_column_to_idx(pos.row, pos.column)
    // }

    // /// Returns None if new pos wouldn't be valid
    // pub fn apply_vec_to_pos(&self, pos: Pos, movement_vec: &MoveVec) -> Option<Pos> {
    //     let x = pos.row as i16 + movement_vec.x;
    //     let y = pos.column as i16 + movement_vec.y;
    //     let column_length = self.board.len() / self.row_length;

    //     if x < 0 || x >= column_length as i16 || y < 0 || y >= self.row_length as i16 {
    //         None
    //     } else {
    //         Some(Pos {
    //             row: x as usize,
    //             column: y as usize,
    //         })
    //     }
    // }

    // /// Checks if applying a move would leave the moving party in check
    // /// If needed, further pre-checks could be run to limit the amounts of moves and pieces evaluated
    // fn legality_check(&self, moves: Vec<LegacyPly>) -> Vec<LegacyPly> {
    //     // if there are no moves, return early
    //     if moves.is_empty() {
    //         return moves;
    //     }

    //     let mut simulation = self.clone();

    //     let mut not_thricefold_repeated = vec![];
    //     for this_move in moves {
    //         simulation.apply_ply(this_move);

    //         // Board position repeated thrice
    //         if self
    //             .repeated_board_count
    //             .get(&simulation.zobrist_hash)
    //             .is_none_or(|&i| i < 3)
    //         {
    //             not_thricefold_repeated.push(this_move);
    //         }

    //         simulation.rewind_last_move();
    //     }
    //     let moves = not_thricefold_repeated;

    //     // if there is no king (i.e. simulations, tests), ignore further legality checks
    //     if self
    //         .piece_map
    //         .get(&(self.next_move_by, PieceType::King))
    //         .is_none()
    //     {
    //         return moves;
    //     }

    //     let mut legal_moves = vec![];
    //     'next: for this_move in moves {
    //         simulation.apply_ply(this_move);

    //         for tile in simulation.board.iter() {
    //             if let Some(piece) = tile {
    //                 // We only care about the other color
    //                 if piece.color != this_move.by.color {
    //                     // We don't really care if the opponents move is legal, just if the king is threatened
    //                     let moves = piece.generate_pseudolegal_moves(&simulation);

    //                     // if any of the pseudolegal moves contain the king's position, it is threatened
    //                     if moves.iter().any(|next_move| {
    //                         simulation
    //                             .piece_map
    //                             .get(&(this_move.by.color, PieceType::King))
    //                             .unwrap()
    //                             .contains(&next_move.move_to.to)
    //                     }) {
    //                         // Rewind the applied move, try next one
    //                         simulation.rewind_last_move();
    //                         continue 'next;
    //                     }
    //                 }
    //             }
    //         }

    //         // No early exit from the tile checking, this move appears to be legal
    //         simulation.rewind_last_move();
    //         legal_moves.push(this_move);
    //     }

    //     legal_moves
    // }

    // pub fn apply_ply(&mut self, ply: LegacyPly) {
    //     // push to simulation list
    //     self.moves.push(ply);

    //     // capture
    //     if let Some(captured_piece) = ply.capturing {
    //         self[captured_piece.pos] = None;
    //     }

    //     // update inner pos on piece
    //     let mut piece = self[ply.move_to.from].unwrap();
    //     piece.pos = ply.move_to.to;
    //     piece.has_moved = true;

    //     // update piece_list mapping
    //     self.piece_map
    //         .entry((piece.color, piece.piece_type))
    //         .and_modify(|pos_list| {
    //             pos_list.retain(|pos| *pos != ply.move_to.from);
    //             pos_list.push(ply.move_to.to)
    //         });

    //     // move piece
    //     self[ply.move_to.from] = None;
    //     self[ply.move_to.to] = Some(piece);
    //     self.next_move_by = self.next_move_by.next();

    //     // Update zobrist hash
    //     self.zobrist_hash = self
    //         .zobrist_table
    //         .update_hash_mailbox(&self, self.zobrist_hash, ply);

    //     // Thricefold repeatition count
    //     *self
    //         .repeated_board_count
    //         .entry(self.zobrist_hash)
    //         .or_insert(0) += 1;
    // }

    // pub fn rewind_last_move(&mut self) {
    //     if let Some(rewind) = self.moves.pop() {
    //         *self
    //             .repeated_board_count
    //             .entry(self.zobrist_hash)
    //             .or_insert(0) -= 1;

    //         // update hash
    //         self.zobrist_hash =
    //             self.zobrist_table
    //                 .update_hash_mailbox(self, self.zobrist_hash, rewind);

    //         // update inner pos on piece
    //         let mut piece = self[rewind.move_to.to].unwrap();
    //         piece.pos = rewind.move_to.from;

    //         // resetting movement flag if this was the first move by piece
    //         if !rewind.by.has_moved {
    //             piece.has_moved = false;
    //         }

    //         // update piece_list mapping
    //         self.piece_map
    //             .entry((piece.color, piece.piece_type))
    //             .and_modify(|pos_list| {
    //                 pos_list.retain(|pos| *pos != rewind.move_to.to);
    //                 pos_list.push(rewind.move_to.from)
    //             });

    //         // move piece back
    //         self[rewind.move_to.to] = None;
    //         self[rewind.move_to.from] = Some(piece);

    //         // reinstantiate captured piece
    //         if let Some(captured_piece) = rewind.capturing {
    //             self[captured_piece.pos] = Some(captured_piece);
    //         }

    //         self.next_move_by = self.next_move_by.next();
    //     }
    // }

    // /// Loosely based on Claude Shannon's evaluation function, adapted for NegaMax
    // fn evaluate(&self) -> i32 {
    //     const KING_WEIGHT: i32 = 4000;
    //     const QUEEN_WEIGHT: i32 = 180;
    //     const ROOK_WEIGHT: i32 = 100;
    //     const BISHOP_WEIGHT: i32 = 60;
    //     const KNIGHT_WEIGHT: i32 = 60;
    //     const PAWN_WEIGHT: i32 = 20;
    //     const ISOLATED_PAWN_WEIGHT: i32 = 5;
    //     const MOVEMENT_WEIGHT: i32 = 1;

    //     // TODO: reweight pawn startegic positions

    //     // We need to:
    //     // - count all pieces
    //     // - count pawns per column per color for doubled and isolated counts
    //     // - count legal moves, and count blocked pawns

    //     let mut piece_counts: HashMap<PieceType, i32> = HashMap::new();
    //     // Entries: (PieceColor::White count, PieceColor::Black count)
    //     let mut white_pawns_per_column = vec![0; self.row_length];
    //     let mut black_pawns_per_column = vec![0; self.row_length];
    //     let mut move_score = 0;
    //     // let mut blocked_pawns = 0;

    //     for tile in self.board.clone().iter() {
    //         if let Some(piece) = tile {
    //             let sign = piece.color.score_sign();

    //             let legal_moves_count = self.get_moves_for_pos(piece.pos).unwrap_or(vec![]).len();

    //             *piece_counts.entry(piece.piece_type).or_insert(0) += sign;
    //             if matches!(piece.piece_type, PieceType::Pawn) {
    //                 match piece.color {
    //                     PieceColor::White => white_pawns_per_column[piece.pos.column] += 1,
    //                     PieceColor::Black => black_pawns_per_column[piece.pos.column] += 1,
    //                 }
    //                 // if legal_moves_count == 0 {
    //                 //     blocked_pawns += sign;
    //                 // }
    //             }

    //             move_score += legal_moves_count as i32 * sign;
    //         }
    //     }

    //     let mut material_score = 0;
    //     for (&piece_type, &count) in piece_counts.iter() {
    //         match piece_type {
    //             PieceType::King => material_score += count * KING_WEIGHT,
    //             PieceType::Queen => material_score += count * QUEEN_WEIGHT,
    //             PieceType::Rook => material_score += count * ROOK_WEIGHT,
    //             PieceType::Bishop => material_score += count * BISHOP_WEIGHT,
    //             PieceType::Knight => material_score += count * KNIGHT_WEIGHT,
    //             PieceType::Pawn => material_score += count * PAWN_WEIGHT,
    //         }
    //     }

    //     // pawns are isolated if they are surrounded by empty columns, so we check both edges and then go through it as a window?
    //     let mut isolated_pawns = 0;
    //     if white_pawns_per_column.len() >= 2 {
    //         // Check left edge
    //         if matches!(
    //             (white_pawns_per_column[0], white_pawns_per_column[1]),
    //             (1, 0)
    //         ) {
    //             isolated_pawns += 1;
    //         }
    //         if matches!(
    //             (black_pawns_per_column[0], black_pawns_per_column[1]),
    //             (1, 0)
    //         ) {
    //             isolated_pawns -= 1;
    //         }

    //         // Check right edge
    //         if matches!(
    //             (
    //                 white_pawns_per_column[white_pawns_per_column.len() - 2],
    //                 white_pawns_per_column[white_pawns_per_column.len() - 1]
    //             ),
    //             (0, 1)
    //         ) {
    //             isolated_pawns += 1;
    //         }
    //         if matches!(
    //             (
    //                 black_pawns_per_column[black_pawns_per_column.len() - 2],
    //                 black_pawns_per_column[black_pawns_per_column.len() - 1]
    //             ),
    //             (0, 1)
    //         ) {
    //             isolated_pawns -= 1;
    //         }
    //     }
    //     for window in white_pawns_per_column.windows(3) {
    //         if matches!(window, &[0, 1, 0]) {
    //             isolated_pawns += 1;
    //         }
    //     }
    //     for window in black_pawns_per_column.windows(3) {
    //         if matches!(window, &[0, 1, 0]) {
    //             isolated_pawns -= 1;
    //         }
    //     }

    //     // let doubled_pawns = white_pawns_per_column
    //     //     .iter()
    //     //     .filter(|&count| *count > 1)
    //     //     .count() as i32
    //     //     - black_pawns_per_column
    //     //         .iter()
    //     //         .filter(|&count| *count > 1)
    //     //         .count() as i32;

    //     // let pawn_score = -((BLOCKED_PAWN_WEIGHT * blocked_pawns)
    //     //     + (DOUBLED_PAWN_WEIGHT * doubled_pawns)
    //     //     + (ISOLATED_PAWN_WEIGHT * isolated_pawns));

    //     let pawn_score = -ISOLATED_PAWN_WEIGHT * isolated_pawns;

    //     (material_score + pawn_score + (MOVEMENT_WEIGHT * move_score))
    //         * self.next_move_by.score_sign()
    // }

    // /// applies MVV-LVA on the unsorted move list
    // pub fn sorted_move_list(&self) -> BinaryHeap<LegacyPly> {
    //     let mut moves = BinaryHeap::new();
    //     for piece in self.board.clone().iter().flatten() {
    //         if piece.color == self.next_move_by {
    //             if let Some(these_moves) = self.get_moves_for_pos(piece.pos) {
    //                 moves.extend(these_moves);
    //             }
    //         }
    //     }
    //     moves
    // }

    // pub fn capturing_move_list(&self) -> Vec<LegacyPly> {
    //     let mut moves = vec![];
    //     for piece in self.board.clone().iter().flatten() {
    //         if piece.color == self.next_move_by {
    //             if let Some(these_moves) = self.get_moves_for_pos(piece.pos) {
    //                 for this_move in these_moves {
    //                     if this_move.capturing.is_some() {
    //                         moves.push(this_move)
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     moves
    // }

    // pub fn search_next_move(&self, depth: i8) -> (i32, Option<LegacyPly>) {
    //     self.alpha_beta(i32::MIN, i32::MAX, depth)
    // }

    // fn quiescence_search(&self, mut alpha: i32, beta: i32) -> i32 {
    //     let eval = self.evaluate();
    //     let mut best_score = eval;

    //     // beta cutoff
    //     if eval >= beta {
    //         return beta;
    //     }
    //     if alpha < eval {
    //         alpha = eval;
    //     }

    //     let mut simulation = self.clone();
    //     for ply in self.capturing_move_list() {
    //         simulation.apply_ply(ply);
    //         let score = simulation
    //             .quiescence_search(beta.saturating_neg(), alpha.saturating_neg())
    //             .saturating_neg();
    //         simulation.rewind_last_move();

    //         if score > best_score {
    //             best_score = score;
    //             if score > alpha {
    //                 alpha = score;
    //             }
    //         }
    //         if score >= beta {
    //             return best_score;
    //         }
    //     }

    //     best_score
    // }

    // fn alpha_beta(&self, mut alpha: i32, beta: i32, depth: i8) -> (i32, Option<LegacyPly>) {
    //     if depth == 0 {
    //         return (
    //             self.quiescence_search(alpha, beta),
    //             self.moves.last().cloned(),
    //         );
    //     };

    //     let mut simulation = self.clone();

    //     let mut best_move = (i32::MIN, None);
    //     for this_move in self.sorted_move_list() {
    //         simulation.apply_ply(this_move);
    //         let score = simulation
    //             .alpha_beta(beta.saturating_neg(), alpha.saturating_neg(), depth - 1)
    //             .0
    //             .saturating_neg();
    //         simulation.rewind_last_move();

    //         if score > best_move.0 {
    //             best_move = (score, Some(this_move));
    //             if score > alpha {
    //                 alpha = score;
    //             }
    //         }
    //         if score >= beta {
    //             return best_move;
    //         }
    //     }
    //     best_move
    // }
}

#[cfg(test)]
mod tests {
    // use crate::chess_engine::moves::MoveTo;

    // use super::*;

    // #[test]
    // fn default_board() {
    //     let game = Game::default();

    //     let mut dict: HashMap<(PieceType, PieceColor), usize> = HashMap::new();

    //     for piece in game.board.iter().flatten() {
    //         if let Some(count) = dict.get_mut(&(piece.piece_type, piece.color)) {
    //             *count += 1;
    //         } else {
    //             dict.insert((piece.piece_type, piece.color), 1);
    //         }
    //     }

    //     assert_eq!(dict.get(&(PieceType::Pawn, PieceColor::White)), Some(&8));
    //     assert_eq!(dict.get(&(PieceType::Pawn, PieceColor::Black)), Some(&8));
    //     assert_eq!(dict.get(&(PieceType::Rook, PieceColor::White)), Some(&2));
    //     assert_eq!(dict.get(&(PieceType::Rook, PieceColor::Black)), Some(&2));
    //     assert_eq!(dict.get(&(PieceType::Bishop, PieceColor::White)), Some(&2));
    //     assert_eq!(dict.get(&(PieceType::Bishop, PieceColor::Black)), Some(&2));
    //     assert_eq!(dict.get(&(PieceType::Knight, PieceColor::White)), Some(&2));
    //     assert_eq!(dict.get(&(PieceType::Knight, PieceColor::Black)), Some(&2));
    //     assert_eq!(dict.get(&(PieceType::Queen, PieceColor::White)), Some(&1));
    //     assert_eq!(dict.get(&(PieceType::Queen, PieceColor::Black)), Some(&1));
    //     assert_eq!(dict.get(&(PieceType::King, PieceColor::White)), Some(&1));
    //     assert_eq!(dict.get(&(PieceType::King, PieceColor::Black)), Some(&1));
    //     assert_eq!(game.board.len(), 8 * 8);
    // }

    // #[test]
    // fn row_and_column_to_idx_test() {
    //     let game = Game {
    //         row_length: 16,
    //         ..Default::default()
    //     };
    //     let result = game.row_and_column_to_idx(0, 0);
    //     assert_eq!(result, 0);

    //     let result = game.row_and_column_to_idx(15, 15);
    //     assert_eq!(result, 255);

    //     let result = game.row_and_column_to_idx(10, 2);
    //     assert_eq!(result, 162);
    // }
    // #[test]
    // fn apply_ply() {
    //     let mut game = Game::new_from_str(
    //         r#"
    //         P0
    //         0p
    //         "#,
    //         2,
    //     );
    //     let dest = Pos::new(0, 0);
    //     let start = Pos::new(1, 1);
    //     let ply = super::LegacyPly {
    //         move_to: MoveTo {
    //             from: start,
    //             to: dest,
    //         },
    //         by: LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::White,
    //             pos: start,
    //             ..Default::default()
    //         },
    //         capturing: Some(LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::Black,
    //             pos: dest,
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     };

    //     game.apply_ply(ply);
    //     assert_eq!(game.board.len(), 4);

    //     assert_eq!(game.moves.len(), 1);
    //     assert_eq!(
    //         game.board[0],
    //         Some(LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::White,
    //             pos: dest,
    //             has_moved: true,
    //             ..Default::default()
    //         })
    //     )
    // }

    // #[test]
    // fn rewind_move() {
    //     let mut game = Game::new_from_str(
    //         r#"
    //         P0
    //         0p
    //         "#,
    //         2,
    //     );
    //     let frozen_board = game.board.clone();
    //     let ply = super::LegacyPly {
    //         move_to: MoveTo {
    //             from: Pos::new(1, 1),
    //             to: Pos::new(0, 0),
    //         },
    //         by: LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::White,
    //             pos: Pos::new(1, 1),
    //             ..Default::default()
    //         },
    //         capturing: Some(LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::Black,
    //             pos: Pos::new(0, 0),
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     };

    //     game.apply_ply(ply);
    //     assert_ne!(game.board, frozen_board);
    //     game.rewind_last_move();
    //     assert_eq!(game.board, frozen_board);
    // }

    // #[test]
    // fn evaluate_default_board() {
    //     let game = Game::default();
    //     let eval = game.evaluate();

    //     // score should be 0 since both sides are fully even
    //     assert_eq!(eval, 0);
    // }

    // #[test]
    // fn evaluate_white_advantage() {
    //     let game = Game::new_from_str(
    //         r#"
    //         RK00
    //         0000
    //         rkr0
    //         "#,
    //         4,
    //     );
    //     let eval = game.evaluate();
    //     assert!(eval.is_positive());
    // }

    // #[test]
    // fn evaluate_pawn_strategic_disadvantages() {
    //     let game = Game::new_from_str(
    //         r#"
    //         PPPPP
    //         00p00
    //         p000p
    //         p000p
    //         "#,
    //         5,
    //     );

    //     let eval = game.evaluate();
    //     assert!(eval.is_negative());
    // }

    // #[test]
    // fn evaluate_movement_disadvantages() {
    //     let game = Game::new_from_str(
    //         r#"
    //         N0000
    //         00000
    //         00n00
    //         00000
    //         00000
    //         "#,
    //         5,
    //     );

    //     let eval = game.evaluate();
    //     assert_ne!(eval, 0);
    //     assert!(eval.is_positive());
    // }

    // #[test]
    // fn evaluate_negamax() {
    //     let mut game = Game::new_from_str(
    //         r#"
    //             N0000
    //             00000
    //             00n00
    //             00000
    //             00000
    //             "#,
    //         5,
    //     );
    //     game.next_move_by = PieceColor::Black;

    //     let eval = game.evaluate();
    //     assert_ne!(eval, 0);
    //     assert!(eval.is_negative());
    // }

    // #[test]
    // fn find_best_move() {
    //     let game = Game::new_from_str(
    //         r#"
    //             R0000r
    //             ppN000
    //             0pp000
    //             000000
    //             000000
    //             "#,
    //         6,
    //     );

    //     let next_move = game.search_next_move(3);
    //     let expected_move = super::LegacyPly {
    //         move_to: MoveTo {
    //             from: Pos::new(1, 1),
    //             to: Pos::new(0, 0),
    //         },
    //         by: LegacyPiece {
    //             piece_type: PieceType::Pawn,
    //             color: PieceColor::White,
    //             pos: Pos::new(1, 1),
    //             ..Default::default()
    //         },
    //         capturing: Some(LegacyPiece {
    //             piece_type: PieceType::Rook,
    //             color: PieceColor::Black,
    //             pos: Pos::new(0, 0),
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     };

    //     assert!(next_move.1.is_some());
    //     assert_eq!(next_move.1, Some(expected_move));
    // }

    // #[test]
    // fn find_best_move_black() {
    //     let mut game = Game::new_from_str(
    //         r#"
    //             R000r
    //             pp000
    //             00p00
    //             00000
    //             00000
    //             "#,
    //         5,
    //     );
    //     game.next_move_by = PieceColor::Black;

    //     let next_move = game.search_next_move(1);
    //     let expected_move = super::LegacyPly {
    //         move_to: MoveTo {
    //             from: Pos::new(0, 0),
    //             to: Pos::new(0, 4),
    //         },
    //         by: LegacyPiece {
    //             piece_type: PieceType::Rook,
    //             color: PieceColor::Black,
    //             pos: Pos::new(0, 0),
    //             ..Default::default()
    //         },
    //         capturing: Some(LegacyPiece {
    //             piece_type: PieceType::Rook,
    //             color: PieceColor::White,
    //             pos: Pos::new(0, 4),
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     };

    //     assert!(next_move.1.is_some());
    //     assert_eq!(next_move.1, Some(expected_move));
    // }

    // #[test]
    // fn quiescence_search_prevents_losses() {
    //     let game = Game::new_from_str(
    //         r#"
    //             R0000Q
    //             00000p
    //             q00000
    //             000000
    //             000000
    //             "#,
    //         6,
    //     );

    //     let next_move = game.search_next_move(1);
    //     let avoided_capture = super::LegacyPly {
    //         move_to: MoveTo {
    //             from: Pos::new(2, 0),
    //             to: Pos::new(0, 0),
    //         },
    //         by: LegacyPiece {
    //             piece_type: PieceType::Queen,
    //             color: PieceColor::White,
    //             pos: Pos::new(2, 0),
    //             ..Default::default()
    //         },
    //         capturing: Some(LegacyPiece {
    //             piece_type: PieceType::Rook,
    //             color: PieceColor::Black,
    //             pos: Pos::new(0, 0),
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     };

    //     assert!(next_move.1.is_some());
    //     assert_ne!(next_move.1, Some(avoided_capture));
    // }

    // #[test]
    // fn thricefold_draw_prevention() {
    //     let mut game = Game::new_from_str(
    //         r#"
    //             0
    //             r
    //             "#,
    //         1,
    //     );

    //     for _ in 0..5 {
    //         let (_, ply) = game.search_next_move(1);
    //         game.apply_ply(ply.unwrap());
    //         game.next_move_by = PieceColor::White;
    //     }

    //     let (_, no_ply) = game.search_next_move(1);
    //     assert!(no_ply.is_none());
    // }
}
