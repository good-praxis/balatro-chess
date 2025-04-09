use std::{cmp::Ordering, collections::BinaryHeap};

use crate::chess_engine::{
    bitboard::{BitIndex, Bitboard, Bitboards, bitboard_idx},
    pieces::{PieceColor, PieceType},
};

/// A classical chess move from either side.
/// contains data for capturing, castling, promotions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Ply {
    pub moving_piece: (PieceType, PieceColor),
    pub from: BitIndex,
    pub to: BitIndex,
    pub capturing: Option<(PieceType, BitIndex)>,
    pub also_move: Option<(PieceType, PieceColor, BitIndex, BitIndex)>,
    pub en_passant_board: Option<Bitboard>,
}

impl PartialOrd for Ply {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // using MVV_LVA (Most Valuable Victim, Least Valuable Attacker)
        match (self.capturing, other.capturing) {
            (None, Some(_)) => return Some(Ordering::Less),
            (Some(_), None) => return Some(Ordering::Greater),
            (None, None) => return Some(self.moving_piece.0.cmp(&other.moving_piece.0)),
            _ => Some(
                self.capture_sorting_value()
                    .cmp(&other.capture_sorting_value()),
            ),
        }
    }
}

impl Ord for Ply {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Ply {
    fn capture_sorting_value(&self) -> u8 {
        if let Some(captured) = self.capturing {
            let victim_value = match captured.0 {
                PieceType::Queen => 25,
                PieceType::Rook => 19,
                PieceType::Bishop => 13,
                PieceType::Knight => 7,
                PieceType::Pawn => 1,
                _ => 0,
            };
            let attacker_value = match self.moving_piece.0 {
                PieceType::Queen => 1,
                PieceType::Rook => 2,
                PieceType::Bishop => 3,
                PieceType::Knight => 4,
                PieceType::Pawn => 5,
                _ => 0,
            };
            return victim_value + attacker_value;
        } else {
            0
        }
    }
}

impl Bitboard {
    /// Returns a iterator of all unblocked single-step plys
    pub fn single_step_plys_in_dirs(
        &self,
        dirs: &[fn(&Self) -> Self],
        blocked: &Self,
        capturable: &Self,
        capturing_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        by_piece: (PieceType, PieceColor),
    ) -> impl Iterator<Item = Ply> {
        dirs.iter()
            .map(|dir| dir(self))
            .filter(|board| **board != 0 && **board & **blocked == 0)
            .map(move |board| {
                let mut capturing = None;
                if *board & **capturable != 0 {
                    // There is a capture present
                    for (piece_type, opposing_board) in capturing_iter.clone() {
                        let capture = board & opposing_board;
                        if *capture != 0 {
                            capturing = Some((piece_type, capture.to_bit_idx()))
                        }
                    }
                }

                Ply {
                    moving_piece: by_piece,
                    from: self.to_bit_idx(),
                    to: board.to_bit_idx(),
                    capturing,
                    ..Default::default()
                }
            })
    }

    /// Returns a iterator of all unblocked multi-step plys (sliding pieces)
    pub fn multi_step_plys_in_dirs(
        &self,
        dirs: &[fn(&Self, &Self, &Self) -> Vec<Self>],
        blocked: &Self,
        capturable: &Self,
        capturing_iter: impl Iterator<Item = (PieceType, Bitboard)> + Clone,
        by_piece: (PieceType, PieceColor),
    ) -> impl Iterator<Item = Ply> {
        dirs.iter()
            .map(|dir| dir(self, blocked, capturable))
            .flatten()
            .map(move |board| {
                let mut capturing = None;
                if *board & **capturable != 0 {
                    // There is a capture present
                    for (piece_type, opposing_board) in capturing_iter.clone() {
                        let capture = board & opposing_board;
                        if *capture != 0 {
                            capturing = Some((piece_type, capture.to_bit_idx()))
                        }
                    }
                }

                Ply {
                    moving_piece: by_piece,
                    from: self.to_bit_idx(),
                    to: board.to_bit_idx(),
                    capturing,
                    ..Default::default()
                }
            })
    }
}

impl Bitboards {
    pub fn make_ply(&mut self, ply: &Ply) {
        // Updating moving piece
        let moving_piece_idx = bitboard_idx(ply.moving_piece.0, ply.moving_piece.1);
        self.boards[moving_piece_idx].set(ply.from, false);
        self.boards[moving_piece_idx].set(ply.to, true);

        // Handle capturing
        if let Some((piece_type, idx)) = ply.capturing {
            // update position boards
            let capturing_idx = bitboard_idx(piece_type, ply.moving_piece.1.next());
            self.boards[capturing_idx].set(idx, false);

            // update piece list
            let piece_list = self.piece_list[capturing_idx]
                .iter()
                .cloned()
                .filter(|this_idx| *this_idx != idx)
                .collect();
            self.piece_list[capturing_idx] = piece_list;
        }

        // Handle linked move
        if let Some((piece_type, piece_color, from, to)) = ply.also_move {
            let moving_piece_idx = bitboard_idx(piece_type, piece_color);
            self.boards[moving_piece_idx].set(from, false);
            self.boards[moving_piece_idx].set(to, true);
        }

        // en passant
        let en_passant = ply.en_passant_board.unwrap_or(0.into());
        self.en_passant = en_passant;

        // update hash
        self.zobrist_hash = self
            .zobrist_table
            .update_hash_bitboard(self.zobrist_hash, ply);
    }

    pub fn unmake_ply(&mut self, ply: &Ply, previous_ply: Option<&Ply>) {
        // Updating moving piece
        let moving_piece_idx = bitboard_idx(ply.moving_piece.0, ply.moving_piece.1);
        self.boards[moving_piece_idx].set(ply.to, false);
        self.boards[moving_piece_idx].set(ply.from, true);

        // Handle capturing
        if let Some((piece_type, idx)) = ply.capturing {
            // update position boards
            let capturing_idx = bitboard_idx(piece_type, ply.moving_piece.1.next());
            self.boards[capturing_idx].set(idx, true);

            // update piece list
            self.piece_list[capturing_idx].push(idx);
        }

        // Handle linked move
        if let Some((piece_type, piece_color, from, to)) = ply.also_move {
            let moving_piece_idx = bitboard_idx(piece_type, piece_color);
            self.boards[moving_piece_idx].set(to, false);
            self.boards[moving_piece_idx].set(from, true);
        }

        // restore en_passant
        if let Some(ply) = previous_ply {
            let en_passant = ply.en_passant_board.unwrap_or(0.into());
            self.en_passant = en_passant;
        } else {
            self.en_passant = 0.into();
        }

        // update hash
        self.zobrist_hash = self
            .zobrist_table
            .update_hash_bitboard(self.zobrist_hash, ply);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::chess_engine::{
        bitboard::{
            Bitboard, Bitboards, bitboard_idx,
            move_gen::{king::KING_DIRS, queen::QUEEN_STEP_DIRS},
        },
        pieces::{PieceColor, PieceType},
    };

    use super::Ply;

    #[test]
    fn single_step_plys() {
        let boards = Bitboards::from_str(
            r#"
            k0
            0P
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::King, PieceColor::White)];

        let mut plys = board
            .single_step_plys_in_dirs(
                &KING_DIRS,
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.all_piece_types_by_color(PieceColor::Black),
                (PieceType::King, PieceColor::White),
            )
            .collect::<BinaryHeap<Ply>>();

        assert_eq!(plys.len(), 3);
        assert!(plys.pop().unwrap().capturing.is_some())
    }

    #[test]
    fn multi_step_plys() {
        let boards = Bitboards::from_str(
            r#"
            q0P
            000
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(PieceType::Queen, PieceColor::White)];

        let mut plys = board
            .multi_step_plys_in_dirs(
                &QUEEN_STEP_DIRS,
                &boards.blocked_mask_for_color(PieceColor::White),
                &boards.all_pieces_by_color(PieceColor::Black),
                boards.all_piece_types_by_color(PieceColor::Black),
                (PieceType::Queen, PieceColor::White),
            )
            .collect::<BinaryHeap<Ply>>();

        assert_eq!(plys.len(), 6);
        assert!(plys.pop().unwrap().capturing.is_some())
    }

    #[test]
    fn mvv_lva() {
        let pawn_takes_pawn = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            capturing: Some((PieceType::Pawn, 0.into())),
            ..Default::default()
        };

        let pawn_takes_queen = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            capturing: Some((PieceType::Queen, 0.into())),
            ..Default::default()
        };

        let queen_takes_pawn = Ply {
            moving_piece: (PieceType::Queen, PieceColor::White),
            capturing: Some((PieceType::Pawn, 0.into())),
            ..Default::default()
        };

        let queen_takes_queen = Ply {
            moving_piece: (PieceType::Queen, PieceColor::White),
            capturing: Some((PieceType::Queen, 0.into())),
            ..Default::default()
        };

        let queen_no_take = Ply {
            moving_piece: (PieceType::Queen, PieceColor::White),
            ..Default::default()
        };

        let pawn_no_take = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            ..Default::default()
        };

        let mut vec = vec![
            pawn_takes_pawn,
            pawn_takes_queen,
            queen_takes_pawn,
            queen_takes_queen,
            queen_no_take,
            pawn_no_take,
        ];
        vec.sort();
        vec.reverse();

        assert_eq!(
            vec,
            vec![
                pawn_takes_queen,
                queen_takes_queen,
                pawn_takes_pawn,
                queen_takes_pawn,
                pawn_no_take,
                queen_no_take,
            ]
        )
    }

    #[test]
    fn make_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        0
        p
        "#,
        );

        let expected = Bitboards::from_str(
            r#"
        p
        0
        "#,
        );

        let ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(bitboard, expected);
    }

    #[test]
    fn unmake_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        0
        p
        "#,
        );

        let expected = bitboard.clone();

        let ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        bitboard.unmake_ply(&ply, None);
        assert_eq!(bitboard, expected);
    }

    #[test]
    fn make_capture_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        0P
        p0
        "#,
        );

        let expected = Bitboards::from_str(
            r#"
        0p
        00
        "#,
        );
        let ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 16.into(),
            to: 1.into(),
            capturing: Some((PieceType::Pawn, 1.into())),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(bitboard, expected);
    }

    #[test]
    fn unmake_capture_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        0
        p
        "#,
        );

        let expected = bitboard.clone();

        let ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 16.into(),
            to: 1.into(),
            capturing: Some((PieceType::Pawn, 1.into())),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        bitboard.unmake_ply(&ply, None);
        assert_eq!(bitboard, expected);
    }

    #[test]
    fn make_en_passant_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        00
        00
        p0
        "#,
        );

        let expected = Bitboards::from_str(
            r#"
        00
        p0
        00
        "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];
        let ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 32.into(),
            to: 0.into(),
            en_passant_board: Some(Bitboard(1 << 16)),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(bitboard.en_passant, expected);
    }

    #[test]
    fn unmake_en_passant_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        00
        00
        p0
        "#,
        );

        let ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 32.into(),
            to: 0.into(),
            en_passant_board: Some(Bitboard(1 << 16)),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        bitboard.unmake_ply(&ply, None);
        assert_eq!(bitboard.en_passant, 0.into());
    }

    #[test]
    fn unmake_en_passant_restore_ply() {
        let mut bitboard = Bitboards::from_str(
            r#"
        00
        00
        p0
        "#,
        );

        let expected = Bitboards::from_str(
            r#"
        00
        p0
        00
        "#,
        );
        let expected = expected.boards[bitboard_idx(PieceType::Pawn, PieceColor::White)];

        let first_ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 32.into(),
            to: 0.into(),
            en_passant_board: Some(Bitboard(1 << 16)),
            ..Default::default()
        };

        let second_ply = Ply {
            moving_piece: (PieceType::Pawn, PieceColor::White),
            from: 0.into(),
            to: 16.into(),
            ..Default::default()
        };

        bitboard.make_ply(&first_ply);
        bitboard.make_ply(&second_ply);
        bitboard.unmake_ply(&second_ply, Some(&first_ply));
        assert_eq!(bitboard.en_passant, expected);
    }
}
