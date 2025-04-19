use ethnum::u256;

use crate::chess_engine::{
    bitboard::{Bitboard, all_pieces_by_color_from_ptr_iter},
    pieces::{Piece, PieceColor, PieceType, PieceWithBitboard},
};

use super::ply::Ply;

fn pawn_dir(color: PieceColor) -> fn(&Bitboard) -> Bitboard {
    if color == PieceColor::White {
        Bitboard::shift_no
    } else {
        Bitboard::shift_so
    }
}

impl Bitboard {
    /// Mask of threatened positions
    pub fn pawn_en_prise_mask(&self, blocked: &Self, color: PieceColor) -> Self {
        let mut mask = Bitboard(u256::ZERO);
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

    pub fn pawn_plys<'a>(
        &'a self,
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        color: PieceColor,
        unmoved_pieces: *const Bitboard,
        en_passant: *const Bitboard,
    ) -> impl Iterator<Item = Ply> {
        let dir = pawn_dir(color);
        let mut moves = vec![];

        let bit_idx = self.to_bit_idx();

        let normal = dir(self);
        if *normal != 0 && *normal & **blocked == 0 && *normal & **capturable == 0 {
            moves.push(Ply {
                moving_piece: Piece(PieceType::Pawn, color),
                from: bit_idx,
                to: normal.to_bit_idx(),
                ..Default::default()
            });

            // Normal push was possible, check for double
            if **self & unsafe { **unmoved_pieces } != 0 {
                let double = dir(&normal);
                if *double != 0 && *double & **blocked == 0 && *normal & **capturable == 0 {
                    moves.push(Ply {
                        moving_piece: Piece(PieceType::Pawn, color),
                        from: bit_idx,
                        to: double.to_bit_idx(),
                        en_passant_board: Some(normal),
                        ..Default::default()
                    });
                }
            }
        }

        // Normal captures
        let capture_dirs = [Bitboard::shift_we, Bitboard::shift_ea];
        for dir in capture_dirs {
            let mut capturing = None;
            let capture = dir(&normal);
            if *capture & **capturable != 0 {
                // There is a capture present
                let capturable_iter = all_pieces_by_color_from_ptr_iter(bitboard_ptr, color.next());
                for PieceWithBitboard(piece_type, opposing_board) in capturable_iter {
                    let capture = capture & opposing_board;
                    if *capture != 0 {
                        capturing = Some((piece_type, capture.to_bit_idx()))
                    }
                }
                moves.push(Ply {
                    moving_piece: Piece(PieceType::Pawn, color),
                    from: bit_idx,
                    to: capture.to_bit_idx(),
                    capturing,
                    ..Default::default()
                })
            }

            // en passant
            if unsafe { **en_passant } != 0 {
                let capture = dir(&normal);
                if *capture & unsafe { **en_passant } != 0 {
                    moves.push(Ply {
                        moving_piece: Piece(PieceType::Pawn, color),
                        from: bit_idx,
                        to: capture.to_bit_idx(),
                        capturing: Some((
                            Piece(PieceType::Pawn, color.next()),
                            pawn_dir(color.next())(&capture).to_bit_idx(),
                        )),
                        ..Default::default()
                    });
                }
            }
        }

        moves.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{Bitboards, Ply, bitboard_idx},
        pieces::{BLACK_PAWN, PieceColor, WHITE_PAWN},
    };

    #[test]
    fn white_pawn_en_prise_mask() {
        let boards = Bitboards::from_str(
            r#"
            000
            0p0
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_PAWN)];

        let expected = Bitboards::from_str(
            r#"
            p0p
            000
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_PAWN)];
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
        let board = boards.boards[bitboard_idx(BLACK_PAWN)];

        let expected = Bitboards::from_str(
            r#"
            000
            p0p
            "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_PAWN)];
        let mask = board.pawn_en_prise_mask(
            &boards.blocked_mask_for_color(PieceColor::Black),
            PieceColor::Black,
        );
        assert_eq!(mask, expected);
    }

    #[test]
    fn pawn_plys() {
        let boards = Bitboards::from_str(
            r#"
            000
            P00
            0p0
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_PAWN)];

        let mut plys: BinaryHeap<Ply> = board
            .pawn_plys(
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.boards.as_ptr(),
                PieceColor::White,
                &boards.unmoved_pieces,
                &boards.en_passant,
            )
            .collect();
        assert_eq!(plys.len(), 3);
        assert!(plys.pop().unwrap().capturing.is_some())
    }

    #[test]
    fn pawn_plys_en_passant() {
        let boards = Bitboards::from_str(
            r#"
            pP0
            000
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(BLACK_PAWN)];

        let en_passant = Bitboards::from_str(
            r#"
            000
            p00
            000
            "#,
        );
        let en_passant = en_passant.boards[bitboard_idx(WHITE_PAWN)];

        let mut plys: BinaryHeap<Ply> = board
            .pawn_plys(
                &boards.blocked_mask_for_color(PieceColor::Black),
                &boards.all_pieces_by_color(PieceColor::White),
                boards.boards.as_ptr(),
                PieceColor::Black,
                &boards.unmoved_pieces,
                &en_passant,
            )
            .collect();
        assert_eq!(plys.len(), 3);
        assert!(plys.pop().unwrap().capturing.is_some())
    }

    #[test]
    fn pawn_cannot_step_on_king() {
        let boards = Bitboards::from_str(
            r#"
            K
            p
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_PAWN)];

        let plys: Vec<Ply> = board
            .pawn_plys(
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.boards.as_ptr(),
                PieceColor::White,
                &boards.unmoved_pieces,
                &boards.en_passant,
            )
            .collect();
        assert_eq!(plys.len(), 0);
    }
}
