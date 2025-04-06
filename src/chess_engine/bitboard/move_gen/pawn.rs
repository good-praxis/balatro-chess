use crate::chess_engine::{bitboard::Bitboard, pieces::PieceColor};

fn pawn_dir(color: PieceColor) -> fn(&Bitboard) -> Bitboard {
    if color == PieceColor::White {
        Bitboard::shift_no
    } else {
        Bitboard::shift_so
    }
}

impl Bitboard {
    /// Cumulative pseudolegal mask of pawn moves
    pub fn pawn_move_mask(
        &self,
        blocked: &Self,
        capturable: &Self,
        color: PieceColor,
        unmoved_pieces: &Self,
        en_passant: &Self,
    ) -> Self {
        self.pawn_move_arr(blocked, capturable, color, unmoved_pieces, en_passant)
            .into_iter()
            .reduce(|acc, e| acc | e)
            .unwrap()
    }
    /// Pseudolegal moves by pawn
    pub fn pawn_move_arr(
        &self,
        blocked: &Self,
        capturable: &Self,
        color: PieceColor,
        unmoved_pieces: &Self,
        en_passant: &Self,
    ) -> Vec<Self> {
        let dir = pawn_dir(color);
        let mut moves = vec![];

        let normal = dir(self);
        if *normal & **blocked == 0 {
            moves.push(normal);

            // Normal push was possible, check for double
            if **self & **unmoved_pieces != 0 {
                let double = dir(&normal);
                if *double & **blocked == 0 {
                    // TODO: en_passant marker
                    moves.push(double);
                }
            }
        }

        // Normal captures
        let capture_one = normal.shift_we();
        if *capture_one & **capturable != 0 {
            // TODO: capture flag
            moves.push(capture_one)
        }

        let capture_two = normal.shift_ea();
        if *capture_two & **capturable != 0 {
            // TODO: capture flag
            moves.push(capture_two)
        }

        // en passant
        if **en_passant != 0 {
            if *capture_one & **en_passant != 0 {
                // TODO: mark captured piece
                moves.push(capture_one);
            }

            if *capture_two & **en_passant != 0 {
                // TODO: mark captured piece
                moves.push(capture_two);
            }
        }

        moves
    }

    /// Mask of threatened positions
    pub fn pawn_en_prise_mask(&self, blocked: &Self, color: PieceColor) -> Self {
        let mut mask = Bitboard(0);
        let normal = pawn_dir(color)(self);
        // Normal captures
        let capture_one = normal.shift_we();
        if *capture_one & **blocked == 0 {
            mask |= capture_one;
        }

        let capture_two = normal.shift_ea();
        if *capture_two & **blocked == 0 {
            mask |= capture_two;
        }

        mask
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{
        bitboard::{Bitboard, Bitboards, bitboard_idx},
        pieces::{PieceColor, PieceType},
    };

    #[test]
    fn white_pawn_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            000
            P00
            0pP
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];

        let en_passant = Bitboards::from_str(
            r#"
            000
            00p
            000
            "#,
        );
        let en_passant = en_passant.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            0p0
            ppp
            000
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];
        let mask = board.pawn_move_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            &boards.all_pieces_by_color(PieceColor::Black),
            PieceColor::White,
            &boards.unmoved_pieces,
            &en_passant,
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn black_pawn_move_mask() {
        let boards = Bitboards::from_str(
            r#"
            0Pp
            p00
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Pawn, PieceColor::Black)];

        let en_passant = Bitboards::from_str(
            r#"
            000
            00p
            000
            "#,
        );
        let en_passant = en_passant.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            000
            ppp
            0p0
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];
        let mask = board.pawn_move_mask(
            &boards.blocked_mask_for_color(PieceColor::Black),
            &boards.all_pieces_by_color(PieceColor::White),
            PieceColor::Black,
            &boards.unmoved_pieces,
            &en_passant,
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn white_pawn_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            000
            0p0
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];

        let expected = Bitboards::from_str(
            r#"
            p0p
            000
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];
        let mask = board.pawn_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::White),
            PieceColor::White,
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn black_pawn_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            0P0
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Pawn, PieceColor::Black)];

        let expected = Bitboards::from_str(
            r#"
            000
            p0p
            "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];
        let mask = board.pawn_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::Black),
            PieceColor::Black,
        );
        assert_eq!(mask, expected);
    }
}
