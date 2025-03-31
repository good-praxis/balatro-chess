use super::moves::{MoveVec, Ply, Pos};
use super::pieces::{Piece, PieceColor, PieceType};
use bevy::prelude::*;
use std::collections::BinaryHeap;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Index, IndexMut},
};

#[derive(Resource, Debug, Clone)]
pub struct Board {
    pub board: Vec<Option<Piece>>,
    pub row_length: usize,
    pub moves: Vec<Ply>,
    pub king_map: HashMap<PieceColor, Pos>,
    pub next_move_by: PieceColor,
}
impl Default for Board {
    fn default() -> Self {
        let row_length = 8;
        Board::from_str(
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
            row_length,
        )
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board_str = String::new();
        for (i, piece) in self.board.iter().enumerate() {
            if i % self.row_length == 0 {
                board_str.push('\n');
            }
            if let Some(piece) = piece {
                board_str.push(piece.to_char());
            } else {
                board_str.push_str("□");
            }
        }
        write!(f, "{}", board_str)
    }
}
impl Index<Pos> for Board {
    type Output = Option<Piece>;

    fn index(&self, pos: Pos) -> &Self::Output {
        &self.board[self.pos_to_idx(pos)]
    }
}
impl IndexMut<Pos> for Board {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let idx = self.pos_to_idx(pos);
        &mut self.board[idx]
    }
}

impl Board {
    pub fn from_str(input: &str, row_length: usize) -> Self {
        let mut board = Vec::new();
        let mut idx = 0;
        let mut king_map = HashMap::new();

        for char in input.chars() {
            // skip whitespaces
            if char.is_whitespace() {
                continue;
            }

            // Empty square
            if char == '0' {
                board.push(None);
                idx += 1;
                continue;
            }

            let row = idx / row_length;
            let column = idx - (row_length * row);
            let pos = Pos::new(row, column);

            let color = if char.is_ascii_lowercase() {
                PieceColor::White
            } else {
                PieceColor::Black
            };
            let piece_type = match char {
                'k' | 'K' => {
                    king_map.insert(color, pos);
                    PieceType::King
                }
                'q' | 'Q' => PieceType::Queen,
                'r' | 'R' => PieceType::Rook,
                'n' | 'N' => PieceType::Knight,
                'b' | 'B' => PieceType::Bishop,
                'p' | 'P' => PieceType::Pawn,
                _ => panic!("Unexpected char: {}", char),
            };

            board.push(Some(Piece::new(piece_type, color, pos)));

            idx += 1;
        }

        Self {
            board,
            row_length,
            moves: vec![],
            king_map,
            next_move_by: PieceColor::White,
        }
    }

    fn get_moves_for_pos(&self, pos: Pos) -> Option<Vec<Ply>> {
        let piece = self[pos];

        if let Some(piece) = piece {
            let moves = piece.generate_pseudolegal_moves(self);
            let legal_moves = self.legality_check(moves);

            if legal_moves.is_empty() {
                None
            } else {
                Some(legal_moves)
            }
        } else {
            None
        }
    }

    /// Takes values from 0 to 15 for the intended board-size
    #[inline]
    pub fn row_and_column_to_idx(&self, row: usize, column: usize) -> usize {
        row * self.row_length + column
    }

    #[inline]
    fn pos_to_idx(&self, pos: Pos) -> usize {
        self.row_and_column_to_idx(pos.row, pos.column)
    }

    /// Returns None if new pos wouldn't be valid
    pub fn apply_vec_to_pos(&self, pos: Pos, movement_vec: &MoveVec) -> Option<Pos> {
        let x = pos.row as i16 + movement_vec.x;
        let y = pos.column as i16 + movement_vec.y;
        let column_length = self.board.len() / self.row_length;

        if x < 0 || x >= column_length as i16 || y < 0 || y >= self.row_length as i16 {
            None
        } else {
            Some(Pos {
                row: x as usize,
                column: y as usize,
            })
        }
    }

    /// Checks if applying a move would leave the moving party in check
    /// If needed, further pre-checks could be run to limit the amounts of moves and pieces evaluated
    fn legality_check(&self, moves: Vec<Ply>) -> Vec<Ply> {
        // if there are no moves, return early
        if moves.is_empty() {
            return moves;
        }

        // if there is no king (i.e. simulations, tests), ignore legality checks
        if self.king_map.get(&moves[0].by.color).is_none() {
            return moves;
        }

        let mut legal_moves = vec![];
        let mut sim_board = self.clone();
        'next: for this_move in moves {
            sim_board.apply_ply(this_move);

            for tile in sim_board.board.iter() {
                if let Some(piece) = tile {
                    // We only care about the other color
                    if piece.color != this_move.by.color {
                        // We don't really care if the opponents move is legal, just if the king is threatened
                        let moves = piece.generate_pseudolegal_moves(&sim_board);

                        // if any of the pseudolegal moves contain the king's position, it is threatened
                        if moves.iter().any(|next_move| {
                            next_move.move_to.to
                                == *sim_board.king_map.get(&this_move.by.color).unwrap()
                        }) {
                            // Rewind the applied move, try next one
                            sim_board.rewind_last_move();
                            continue 'next;
                        }
                    }
                }
            }

            // No early exit from the tile checking, this move appears to be legal
            sim_board.rewind_last_move();
            legal_moves.push(this_move);
        }

        legal_moves
    }

    pub fn apply_ply(&mut self, ply: Ply) {
        // push to simulation list
        self.moves.push(ply);

        // capture
        if let Some(captured_piece) = ply.capturing {
            self[captured_piece.pos] = None;
        }

        // update inner pos on piece
        let mut piece = self[ply.move_to.from].unwrap();
        piece.pos = ply.move_to.to;

        // if piece is king, we also need to move it's mapping
        if piece.piece_type == PieceType::King {
            self.king_map.insert(piece.color, ply.move_to.to);
        }

        // move piece
        self[ply.move_to.from] = None;
        self[ply.move_to.to] = Some(piece);
        self.next_move_by = self.next_move_by.next();
    }

    pub fn rewind_last_move(&mut self) {
        if let Some(rewind) = self.moves.pop() {
            // update inner pos on piece
            let mut piece = self[rewind.move_to.to].unwrap();
            piece.pos = rewind.move_to.from;

            // if piece is king, we also need to move it's mapping
            if piece.piece_type == PieceType::King {
                self.king_map.insert(piece.color, rewind.move_to.from);
            }

            // move piece back
            self[rewind.move_to.to] = None;
            self[rewind.move_to.from] = Some(piece);

            // reinstantiate captured piece
            if let Some(captured_piece) = rewind.capturing {
                self[captured_piece.pos] = Some(captured_piece);
            }

            self.next_move_by = self.next_move_by.next();
        }
    }

    /// Loosely based on Claude Shannon's evaluation function, adapted for NegaMax
    fn evaluate(&self) -> i32 {
        const KING_WEIGHT: i32 = 4000;
        const QUEEN_WEIGHT: i32 = 180;
        const ROOK_WEIGHT: i32 = 100;
        const BISHOP_WEIGHT: i32 = 60;
        const KNIGHT_WEIGHT: i32 = 60;
        const PAWN_WEIGHT: i32 = 20;
        const ISOLATED_PAWN_WEIGHT: i32 = 5;
        const MOVEMENT_WEIGHT: i32 = 1;

        // TODO: reweight pawn startegic positions

        // We need to:
        // - count all pieces
        // - count pawns per column per color for doubled and isolated counts
        // - count legal moves, and count blocked pawns

        let mut piece_counts: HashMap<PieceType, i32> = HashMap::new();
        // Entries: (PieceColor::White count, PieceColor::Black count)
        let mut white_pawns_per_column = vec![0; self.row_length];
        let mut black_pawns_per_column = vec![0; self.row_length];
        let mut move_score = 0;
        // let mut blocked_pawns = 0;

        for tile in self.board.clone().iter() {
            if let Some(piece) = tile {
                let sign = piece.color.score_sign();

                let legal_moves_count = self.get_moves_for_pos(piece.pos).unwrap_or(vec![]).len();

                *piece_counts.entry(piece.piece_type).or_insert(0) += sign;
                if matches!(piece.piece_type, PieceType::Pawn) {
                    match piece.color {
                        PieceColor::White => white_pawns_per_column[piece.pos.column] += 1,
                        PieceColor::Black => black_pawns_per_column[piece.pos.column] += 1,
                    }
                    // if legal_moves_count == 0 {
                    //     blocked_pawns += sign;
                    // }
                }

                move_score += legal_moves_count as i32 * sign;
            }
        }

        let mut material_score = 0;
        for (&piece_type, &count) in piece_counts.iter() {
            match piece_type {
                PieceType::King => material_score += count * KING_WEIGHT,
                PieceType::Queen => material_score += count * QUEEN_WEIGHT,
                PieceType::Rook => material_score += count * ROOK_WEIGHT,
                PieceType::Bishop => material_score += count * BISHOP_WEIGHT,
                PieceType::Knight => material_score += count * KNIGHT_WEIGHT,
                PieceType::Pawn => material_score += count * PAWN_WEIGHT,
            }
        }

        // pawns are isolated if they are surrounded by empty columns, so we check both edges and then go through it as a window?
        let mut isolated_pawns = 0;
        if white_pawns_per_column.len() >= 2 {
            // Check left edge
            if matches!(
                (white_pawns_per_column[0], white_pawns_per_column[1]),
                (1, 0)
            ) {
                isolated_pawns += 1;
            }
            if matches!(
                (black_pawns_per_column[0], black_pawns_per_column[1]),
                (1, 0)
            ) {
                isolated_pawns -= 1;
            }

            // Check right edge
            if matches!(
                (
                    white_pawns_per_column[white_pawns_per_column.len() - 2],
                    white_pawns_per_column[white_pawns_per_column.len() - 1]
                ),
                (0, 1)
            ) {
                isolated_pawns += 1;
            }
            if matches!(
                (
                    black_pawns_per_column[black_pawns_per_column.len() - 2],
                    black_pawns_per_column[black_pawns_per_column.len() - 1]
                ),
                (0, 1)
            ) {
                isolated_pawns -= 1;
            }
        }
        for window in white_pawns_per_column.windows(3) {
            if matches!(window, &[0, 1, 0]) {
                isolated_pawns += 1;
            }
        }
        for window in black_pawns_per_column.windows(3) {
            if matches!(window, &[0, 1, 0]) {
                isolated_pawns -= 1;
            }
        }

        let doubled_pawns = white_pawns_per_column
            .iter()
            .filter(|&count| *count > 1)
            .count() as i32
            - black_pawns_per_column
                .iter()
                .filter(|&count| *count > 1)
                .count() as i32;

        // let pawn_score = -((BLOCKED_PAWN_WEIGHT * blocked_pawns)
        //     + (DOUBLED_PAWN_WEIGHT * doubled_pawns)
        //     + (ISOLATED_PAWN_WEIGHT * isolated_pawns));

        let pawn_score = -ISOLATED_PAWN_WEIGHT * isolated_pawns;

        (material_score + pawn_score + (MOVEMENT_WEIGHT * move_score))
            * self.next_move_by.score_sign()
    }

    pub fn unsorted_move_list(&self) -> Vec<Ply> {
        let mut moves = vec![];
        for piece in self.board.clone().iter().flatten() {
            if piece.color == self.next_move_by {
                if let Some(these_moves) = self.get_moves_for_pos(piece.pos) {
                    moves.extend(these_moves);
                }
            }
        }
        moves
    }

    /// applies MVV-LVA on the unsorted move list
    pub fn sorted_move_list(&self) -> BinaryHeap<Ply> {
        let mut moves = BinaryHeap::new();
        for piece in self.board.clone().iter().flatten() {
            if piece.color == self.next_move_by {
                if let Some(these_moves) = self.get_moves_for_pos(piece.pos) {
                    moves.extend(these_moves);
                }
            }
        }
        moves
    }

    pub fn capturing_move_list(&self) -> Vec<Ply> {
        let mut moves = vec![];
        for piece in self.board.clone().iter().flatten() {
            if piece.color == self.next_move_by {
                if let Some(these_moves) = self.get_moves_for_pos(piece.pos) {
                    for this_move in these_moves {
                        if this_move.capturing.is_some() {
                            moves.push(this_move)
                        }
                    }
                }
            }
        }
        moves
    }

    pub fn search_next_move(&self, depth: i8) -> (i32, Option<Ply>) {
        self.alpha_beta(i32::MIN, i32::MAX, depth)
    }

    fn quiescence_search(&self, mut alpha: i32, beta: i32) -> i32 {
        let eval = self.evaluate();
        let mut best_score = eval;

        // beta cutoff
        if eval >= beta {
            return beta;
        }
        if alpha < eval {
            alpha = eval;
        }

        let mut sim_board = self.clone();
        for ply in self.capturing_move_list() {
            sim_board.apply_ply(ply);
            let score = sim_board
                .quiescence_search(beta.saturating_neg(), alpha.saturating_neg())
                .saturating_neg();
            sim_board.rewind_last_move();

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                }
            }
            if score >= beta {
                return best_score;
            }
        }

        best_score
    }

    fn alpha_beta(&self, mut alpha: i32, beta: i32, depth: i8) -> (i32, Option<Ply>) {
        if depth == 0 {
            return (
                self.quiescence_search(alpha, beta),
                self.moves.last().cloned(),
            );
        };

        let mut sim_board = self.clone();

        let mut best_move = (i32::MIN, None);
        for this_move in self.sorted_move_list() {
            sim_board.apply_ply(this_move);
            let score = sim_board
                .alpha_beta(beta.saturating_neg(), alpha.saturating_neg(), depth - 1)
                .0
                .saturating_neg();
            sim_board.rewind_last_move();

            if score > best_move.0 {
                best_move = (score, Some(this_move));
                if score > alpha {
                    alpha = score;
                }
            }
            if score >= beta {
                return best_move;
            }
        }
        best_move
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::moves::MoveTo;

    use super::*;

    #[test]
    fn default_board() {
        let board = Board::default();

        let mut dict: HashMap<(PieceType, PieceColor), usize> = HashMap::new();

        for piece in board.board.iter().flatten() {
            if let Some(count) = dict.get_mut(&(piece.piece_type, piece.color)) {
                *count += 1;
            } else {
                dict.insert((piece.piece_type, piece.color), 1);
            }
        }

        assert_eq!(dict.get(&(PieceType::Pawn, PieceColor::White)), Some(&8));
        assert_eq!(dict.get(&(PieceType::Pawn, PieceColor::Black)), Some(&8));
        assert_eq!(dict.get(&(PieceType::Rook, PieceColor::White)), Some(&2));
        assert_eq!(dict.get(&(PieceType::Rook, PieceColor::Black)), Some(&2));
        assert_eq!(dict.get(&(PieceType::Bishop, PieceColor::White)), Some(&2));
        assert_eq!(dict.get(&(PieceType::Bishop, PieceColor::Black)), Some(&2));
        assert_eq!(dict.get(&(PieceType::Knight, PieceColor::White)), Some(&2));
        assert_eq!(dict.get(&(PieceType::Knight, PieceColor::Black)), Some(&2));
        assert_eq!(dict.get(&(PieceType::Queen, PieceColor::White)), Some(&1));
        assert_eq!(dict.get(&(PieceType::Queen, PieceColor::Black)), Some(&1));
        assert_eq!(dict.get(&(PieceType::King, PieceColor::White)), Some(&1));
        assert_eq!(dict.get(&(PieceType::King, PieceColor::Black)), Some(&1));
        assert_eq!(board.board.len(), 8 * 8);
    }

    #[test]
    fn row_and_column_to_idx_test() {
        let board = Board {
            board: vec![],
            row_length: 16,
            moves: vec![],
            king_map: HashMap::new(),
            next_move_by: PieceColor::White,
        };
        let result = board.row_and_column_to_idx(0, 0);
        assert_eq!(result, 0);

        let result = board.row_and_column_to_idx(15, 15);
        assert_eq!(result, 255);

        let result = board.row_and_column_to_idx(10, 2);
        assert_eq!(result, 162);
    }
    #[test]
    fn apply_ply() {
        let mut board = Board::from_str(
            r#"
            P0
            0p
            "#,
            2,
        );
        let dest = Pos::new(0, 0);
        let start = Pos::new(1, 1);
        let ply = super::Ply {
            move_to: MoveTo {
                from: start,
                to: dest,
            },
            by: Piece {
                piece_type: PieceType::Pawn,
                color: PieceColor::White,
                pos: start,
                ..Default::default()
            },
            capturing: Some(Piece {
                piece_type: PieceType::Pawn,
                color: PieceColor::Black,
                pos: dest,
                ..Default::default()
            }),
            ..Default::default()
        };

        board.apply_ply(ply);
        assert_eq!(board.board.len(), 4);

        assert_eq!(board.moves.len(), 1);
        assert_eq!(
            board.board[0],
            Some(Piece {
                piece_type: PieceType::Pawn,
                color: PieceColor::White,
                pos: dest,
                ..Default::default()
            })
        )
    }

    #[test]
    fn rewind_move() {
        let mut board = Board::from_str(
            r#"
            P0
            0p
            "#,
            2,
        );
        let frozen_board = board.board.clone();
        let ply = super::Ply {
            move_to: MoveTo {
                from: Pos::new(1, 1),
                to: Pos::new(0, 0),
            },
            by: Piece {
                piece_type: PieceType::Pawn,
                color: PieceColor::White,
                pos: Pos::new(1, 1),
                ..Default::default()
            },
            capturing: Some(Piece {
                piece_type: PieceType::Pawn,
                color: PieceColor::Black,
                pos: Pos::new(0, 0),
                ..Default::default()
            }),
            ..Default::default()
        };

        board.apply_ply(ply);
        assert_ne!(board.board, frozen_board);
        board.rewind_last_move();
        assert_eq!(board.board, frozen_board);
    }

    #[test]
    fn evaluate_default_board() {
        let board = Board::default();
        let eval = board.evaluate();

        // score should be 0 since both sides are fully even
        assert_eq!(eval, 0);
    }

    #[test]
    fn evaluate_white_advantage() {
        let board = Board::from_str(
            r#"
            RK00
            0000
            rkr0
            "#,
            4,
        );
        let eval = board.evaluate();
        assert!(eval.is_positive());
    }

    #[test]
    fn evaluate_pawn_strategic_disadvantages() {
        let board = Board::from_str(
            r#"
            PPPPP
            00p00
            p000p
            p000p
            "#,
            5,
        );

        let eval = board.evaluate();
        assert!(eval.is_negative());
    }

    #[test]
    fn evaluate_movement_disadvantages() {
        let board = Board::from_str(
            r#"
            N0000
            00000
            00n00
            00000
            00000
            "#,
            5,
        );

        let eval = board.evaluate();
        assert_ne!(eval, 0);
        assert!(eval.is_positive());
    }

    #[test]
    fn evaluate_negamax() {
        let mut board = Board::from_str(
            r#"
                N0000
                00000
                00n00
                00000
                00000
                "#,
            5,
        );
        board.next_move_by = PieceColor::Black;

        let eval = board.evaluate();
        assert_ne!(eval, 0);
        assert!(eval.is_negative());
    }

    #[test]
    fn find_best_move() {
        let board = Board::from_str(
            r#"
                R0000r
                ppN000
                0pp000
                000000
                000000
                "#,
            6,
        );

        let next_move = board.search_next_move(3);
        let expected_move = super::Ply {
            move_to: MoveTo {
                from: Pos::new(1, 1),
                to: Pos::new(0, 0),
            },
            by: Piece {
                piece_type: PieceType::Pawn,
                color: PieceColor::White,
                pos: Pos::new(1, 1),
                ..Default::default()
            },
            capturing: Some(Piece {
                piece_type: PieceType::Rook,
                color: PieceColor::Black,
                pos: Pos::new(0, 0),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert!(next_move.1.is_some());
        assert_eq!(next_move.1, Some(expected_move));
    }

    #[test]
    fn find_best_move_black() {
        let mut board = Board::from_str(
            r#"
                R000r
                pp000
                00p00
                00000
                00000
                "#,
            5,
        );
        board.next_move_by = PieceColor::Black;

        let next_move = board.search_next_move(1);
        let expected_move = super::Ply {
            move_to: MoveTo {
                from: Pos::new(0, 0),
                to: Pos::new(0, 4),
            },
            by: Piece {
                piece_type: PieceType::Rook,
                color: PieceColor::Black,
                pos: Pos::new(0, 0),
                ..Default::default()
            },
            capturing: Some(Piece {
                piece_type: PieceType::Rook,
                color: PieceColor::White,
                pos: Pos::new(0, 4),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert!(next_move.1.is_some());
        assert_eq!(next_move.1, Some(expected_move));
    }

    #[test]
    fn quiescence_search_prevents_losses() {
        let board = Board::from_str(
            r#"
                R0000Q
                00000p
                q00000
                000000
                000000
                "#,
            6,
        );

        let next_move = board.search_next_move(1);
        let avoided_capture = super::Ply {
            move_to: MoveTo {
                from: Pos::new(2, 0),
                to: Pos::new(0, 0),
            },
            by: Piece {
                piece_type: PieceType::Queen,
                color: PieceColor::White,
                pos: Pos::new(2, 0),
                ..Default::default()
            },
            capturing: Some(Piece {
                piece_type: PieceType::Rook,
                color: PieceColor::Black,
                pos: Pos::new(0, 0),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert!(next_move.1.is_some());
        assert_ne!(next_move.1, Some(avoided_capture));
    }
}
