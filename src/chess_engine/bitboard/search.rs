use strum::IntoEnumIterator;

use crate::chess_engine::{
    bitboard::Ply,
    pieces::{BLACK_PAWN, Piece, PieceColor, PieceType},
};
use std::collections::BinaryHeap;

use super::{Bitboards, bitboard_idx};

#[derive(Debug)]
pub struct Weights {
    // Material weights
    pub king: i32,
    pub queen: i32,
    pub rook: i32,
    pub bishop: i32,
    pub knight: i32,
    pub pawn: i32,

    // Strategic weights
    pub isolated_pawn: i32,
    pub movement: i32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            king: 4000,
            queen: 180,
            rook: 100,
            bishop: 60,
            knight: 60,
            pawn: 20,
            isolated_pawn: -5,
            movement: 1,
        }
    }
}

/// Metadata stuct for search
#[derive(Default, Debug)]
pub struct SearchMeta {
    current_tree: Vec<Ply>,
    nodes_visited: u64,
    /// Index: WeightMap
    weights: Weights,
    // PV
    follow_pv: bool,
    score_pv: bool,
}
impl SearchMeta {
    fn with_weights(weights: Weights) -> Self {
        Self {
            weights,
            ..Default::default()
        }
    }

    fn last_ply_by(&self) -> PieceColor {
        self.current_tree
            .last()
            .unwrap_or(&Ply {
                moving_piece: BLACK_PAWN, // Default to Black, since White moves first
                ..Default::default()
            })
            .moving_piece
            .1
    }
}

impl Bitboards {
    pub fn evaluate(&self, meta: &SearchMeta) -> i32 {
        if self.check_cache {
            if let Some(eval) = self
                .evaluation_table
                .lock()
                .unwrap()
                .get(&self.zobrist_hash)
            {
                return *eval;
            }
        }

        // TODO: reweight pawn startegic positions
        // TODO: Add strategic weight of pawns

        // We need to:
        // - count all pieces
        // - count pawns per column per color for doubled and isolated counts
        // - count legal moves, and count blocked pawns

        // Material score
        let material_score: i32 = self
            .key_value_pieces_iter()
            .map(|(piece, _)| match piece {
                Piece(PieceType::King, color) => color.score_sign() * meta.weights.king,
                Piece(PieceType::Queen, color) => color.score_sign() * meta.weights.queen,
                Piece(PieceType::Rook, color) => color.score_sign() * meta.weights.rook,
                Piece(PieceType::Bishop, color) => color.score_sign() * meta.weights.bishop,
                Piece(PieceType::Knight, color) => color.score_sign() * meta.weights.knight,
                Piece(PieceType::Pawn, color) => color.score_sign() * meta.weights.pawn,
            })
            .sum();

        // Isolate pawn count
        let window: u16 = 0b010;
        let mask: u16 = 0b111;
        let mut isolated_pawns_count = 0;

        for color in PieceColor::iter() {
            let pawns =
                self.boards[bitboard_idx(Piece(PieceType::Pawn, color))].to_column_representation();

            if pawns.trailing_ones() == 1 {
                isolated_pawns_count += color.score_sign()
            }

            if pawns.leading_ones() == 1 {
                isolated_pawns_count += color.score_sign()
            }

            for _ in 0..=13 {
                let masked_pawns = pawns & mask;
                if (masked_pawns | window).count_ones() == 1 {
                    isolated_pawns_count += color.score_sign()
                }
            }
        }

        let pawn_score = meta.weights.isolated_pawn * isolated_pawns_count;

        // Move score
        let move_score = self
            .all_legal_plys_by_color::<Vec<Ply>>(PieceColor::White)
            .len() as i32
            - self
                .all_legal_plys_by_color::<Vec<Ply>>(PieceColor::Black)
                .len() as i32;

        let score = (material_score + pawn_score + (meta.weights.movement * move_score))
            * meta.last_ply_by().next().score_sign();

        self.evaluation_table
            .lock()
            .unwrap()
            .insert(*self.zobrist_hash, score);
        score
    }

    fn quiescence_search(&mut self, meta: &mut SearchMeta, mut alpha: i32, beta: i32) -> i32 {
        // Check cached results
        if self.check_cache {
            if let Some(result) = self
                .quiescence_table
                .lock()
                .unwrap()
                .get(&self.zobrist_hash)
            {
                return *result;
            }
        }

        let eval = self.evaluate(meta);
        let mut best_score = eval;

        //beta cutoff
        if eval >= beta {
            return beta;
        }

        if alpha < eval {
            alpha = eval;
        }

        for ply in self.all_legal_capturing_plys_by_color::<Vec<Ply>>(meta.last_ply_by().next()) {
            meta.nodes_visited += 1;
            self.make_ply(&ply);
            meta.current_tree.push(ply);

            let score = self
                .quiescence_search(meta, beta.saturating_neg(), alpha.saturating_neg())
                .saturating_neg();
            let last_ply = meta.current_tree.pop().unwrap_or_default();
            self.unmake_ply(&last_ply, meta.current_tree.last());

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                }
            }
            if score >= beta {
                break;
            }
        }

        self.quiescence_table
            .lock()
            .unwrap()
            .insert(*self.zobrist_hash, best_score);
        best_score
    }

    fn alpha_beta(
        &mut self,
        meta: &mut SearchMeta,
        mut alpha: i32,
        beta: i32,
        depth: i8,
    ) -> (i32, Option<Ply>) {
        if depth == 0 {
            return (
                self.quiescence_search(meta, alpha, beta),
                meta.current_tree.last().cloned(),
            );
        };

        let mut best_move = (i32::MIN, None);

        let mut priority_queue =
            self.all_legal_plys_by_color::<BinaryHeap<Ply>>(meta.last_ply_by().next());

        // PV following
        if meta.follow_pv {
            meta.follow_pv = false;
            if let Some(&pv) = self.pv_table.lock().unwrap().get(&self.zobrist_hash) {
                meta.follow_pv = true;
                priority_queue.push(pv);
            }
        }

        for this_move in priority_queue {
            meta.nodes_visited += 1;
            self.make_ply(&this_move);
            meta.current_tree.push(this_move);
            let score = self
                .alpha_beta(
                    meta,
                    beta.saturating_neg(),
                    alpha.saturating_neg(),
                    depth - 1,
                )
                .0
                .saturating_neg();
            let last_ply = meta.current_tree.pop().unwrap_or_default();
            self.unmake_ply(&last_ply, meta.current_tree.last());

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
        if let Some(mut pv) = best_move.1 {
            pv.pv_move = true;
            self.pv_table.lock().unwrap().insert(*self.zobrist_hash, pv);
        }

        best_move
    }

    /// Searches the next best ply at a given depth + quienscence search;
    /// Returns the (score, best_ply, visited_nodes_count)
    pub fn search_next_ply(
        &mut self,
        last_ply: Option<Ply>,
        depth: i8,
        weights: Weights,
    ) -> (i32, Option<Ply>, u64) {
        let mut meta = SearchMeta::with_weights(weights);
        if last_ply.is_some() {
            meta.current_tree.push(last_ply.unwrap());
        }
        let result = self.iterative_deepening(&mut meta, depth);
        (result.0, result.1, meta.nodes_visited)
    }

    pub fn iterative_deepening(&mut self, meta: &mut SearchMeta, depth: i8) -> (i32, Option<Ply>) {
        let mut result = (0, None);
        for i in 1..=depth {
            meta.follow_pv = true;
            result = self.alpha_beta(meta, i32::MIN, i32::MAX, i);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::i32::{MAX, MIN};

    use super::*;
    use crate::chess_engine::{game::Game, pieces::WHITE_ROOK};

    #[test]
    fn evaluate_default() {
        let game = Game::default();
        let boards = game.boards;
        let score = boards.evaluate(&SearchMeta::default());
        assert_eq!(score, 0);
    }

    #[test]
    fn evaluate_material_score() {
        let boards = Bitboards::from_str(
            r#"
            ppP
            PPP
            "#,
        );
        let score = boards.evaluate(&SearchMeta::default());
        assert!(score.is_negative());
    }

    #[test]
    fn evaluate_movement_score() {
        let boards = Bitboards::from_str(
            r#"
            00000
            00000
            00q00
            00000
            0000Q
            "#,
        );
        let score = boards.evaluate(&SearchMeta::default());
        assert!(score.is_positive());
    }

    #[test]
    fn evaluate_isolated_pawns_score() {
        let boards = Bitboards::from_str(
            r#"
            PPPPPPPP
            00000000
            00000000
            p0p0p0pp
            00000000
            p000p00p
            "#,
        );
        let score = boards.evaluate(&SearchMeta::default());
        assert!(score.is_negative());
    }

    #[test]
    fn quiescence_search_until_quiet_position() {
        let mut boards = Bitboards::from_str(
            r#"
            P0P
            0P0
            p0p
            "#,
        );
        let mut meta = SearchMeta::default();
        let _score = boards.quiescence_search(&mut meta, MIN, MAX);
        assert_eq!(meta.nodes_visited, 8);
    }

    #[test]
    fn alpha_beta_search_nodes_visited() {
        let mut boards = Bitboards::from_str(
            r#"
            0QR
            q00
            0r0
            "#,
        );
        let mut meta = SearchMeta::default();
        let _score = boards.alpha_beta(&mut meta, MIN, MAX, 1);
        assert_eq!(meta.nodes_visited, 11);
    }

    #[test]
    fn alpha_beta_search_expected_result() {
        let mut boards = Bitboards::from_str(
            r#"
            0QR
            q00
            0r0
            "#,
        );
        let mut meta = SearchMeta::default();
        let result = boards.alpha_beta(&mut meta, MIN, MAX, 1);
        assert!(result.1.is_some());
        assert_eq!(result.1.unwrap().moving_piece, WHITE_ROOK)
    }

    #[test]
    fn checkmate_search() {
        let mut boards = Bitboards::from_str(
            r#"
            kR0
            0R0
            0r0
            "#,
        );
        let result = boards.search_next_ply(None, 3, Weights::default());
        assert!(result.1.is_none());
    }

    #[test]
    fn pre_checkmate_search() {
        let mut boards = Bitboards::from_str(
            r#"
            KRr
            0r0
            0R0
            "#,
        );
        let result = boards.search_next_ply(None, 3, Weights::default());
        assert!(result.1.is_some());
        let ply = result.1;
        boards.make_ply(&ply.unwrap());
        let result = boards.search_next_ply(ply, 3, Weights::default());
        assert!(result.1.is_none());
    }

    #[test]
    fn iterative_deepening_pv_trim_nodes() {
        let mut boards = Game::default().boards;

        let mut iterative_meta = SearchMeta::default();
        let _iterative = boards.iterative_deepening(&mut iterative_meta, 3);

        let mut exhaustive_meta = SearchMeta::default();
        let _exhaustive = boards.alpha_beta(&mut exhaustive_meta, MIN, MAX, 3);

        assert!(iterative_meta.nodes_visited < exhaustive_meta.nodes_visited);
    }
}
