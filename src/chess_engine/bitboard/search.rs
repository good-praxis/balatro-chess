use crate::chess_engine::{
    bitboard::Ply,
    pieces::{PieceColor, PieceType},
};

use super::Bitboards;

/// Metadata stuct for search
#[derive(Default, Clone, Debug)]
struct SearchMeta {
    current_tree: Vec<Ply>,
}
impl SearchMeta {
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
}

#[cfg(test)]
mod tests {
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
}
