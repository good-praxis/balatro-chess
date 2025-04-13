use crate::chess_engine::{
    bitboard::Ply,
    pieces::{PieceColor, PieceType},
};
use std::collections::BinaryHeap;

use super::Bitboards;

#[derive(Debug, Default)]
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

/// Metadata stuct for search
#[derive(Default, Debug)]
struct SearchMeta {
    current_tree: Vec<Ply>,
    nodes_visited: u64,
    /// Index: WeightMap
    weights: Weights,
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
                moving_piece: (PieceType::Pawn, PieceColor::Black), // Default to Black, since White moves first
                ..Default::default()
            })
            .moving_piece
            .1
    }
}

impl Bitboards {
    pub fn evaluate(&self, search_meta: &SearchMeta) -> i32 {
        const KING_WEIGHT: i32 = 4000;
        const QUEEN_WEIGHT: i32 = 180;
        const ROOK_WEIGHT: i32 = 100;
        const BISHOP_WEIGHT: i32 = 60;
        const KNIGHT_WEIGHT: i32 = 60;
        const PAWN_WEIGHT: i32 = 20;
        const MOVEMENT_WEIGHT: i32 = 1;

        // TODO: reweight pawn startegic positions
        // TODO: Add strategic weight of pawns

        // We need to:
        // - count all pieces
        // - count pawns per column per color for doubled and isolated counts
        // - count legal moves, and count blocked pawns

        let material_score: i32 = self
            .key_value_pieces_iter()
            .map(|((piece_type, piece_color), _)| match piece_type {
                PieceType::King => piece_color.score_sign() * KING_WEIGHT,
                PieceType::Queen => piece_color.score_sign() * QUEEN_WEIGHT,
                PieceType::Rook => piece_color.score_sign() * ROOK_WEIGHT,
                PieceType::Bishop => piece_color.score_sign() * BISHOP_WEIGHT,
                PieceType::Knight => piece_color.score_sign() * KNIGHT_WEIGHT,
                PieceType::Pawn => piece_color.score_sign() * PAWN_WEIGHT,
            })
            .sum();

        let move_score = self
            .all_legal_plys_by_color::<Vec<Ply>>(PieceColor::White)
            .len() as i32
            - self
                .all_legal_plys_by_color::<Vec<Ply>>(PieceColor::Black)
                .len() as i32;

        (material_score + (MOVEMENT_WEIGHT * move_score))
            * search_meta.last_ply_by().next().score_sign()
    }

    fn quiescence_search(&mut self, meta: &mut SearchMeta, mut alpha: i32, beta: i32) -> i32 {
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
                return best_score;
            }
        }

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
        for this_move in self.all_legal_plys_by_color::<BinaryHeap<Ply>>(meta.last_ply_by().next())
        {
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
        let result = self.alpha_beta(&mut meta, i32::MIN, i32::MAX, depth);
        (result.0, result.1, meta.nodes_visited)
    }
}

#[cfg(test)]
mod tests {
    use std::i32::{MAX, MIN};

    use super::*;
    use crate::chess_engine::game::Game;

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
        assert_eq!(
            result.1.unwrap().moving_piece,
            (PieceType::Rook, PieceColor::White)
        )
    }
}
