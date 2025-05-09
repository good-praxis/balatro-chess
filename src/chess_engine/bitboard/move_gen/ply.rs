use ethnum::u256;

use crate::chess_engine::{
    bitboard::{BitIndex, Bitboard, Bitboards, all_pieces_by_color_from_ptr_iter, bitboard_idx},
    pieces::{Piece, PieceColor, PieceType, PieceWithBitboard},
};
use std::{cmp::Ordering, fmt::Display};

/// A classical chess move from either side.
/// contains data for capturing, castling, promotions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Ply {
    pub moving_piece: Piece,
    pub from: BitIndex,
    pub to: BitIndex,
    pub capturing: Option<(Piece, BitIndex)>,
    pub also_move: Option<(Piece, BitIndex, BitIndex)>,
    pub en_passant_board: Option<Bitboard>,
    pub pv_move: bool,
}

impl Display for Ply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = self.moving_piece.as_char();
        let from = self.from.to_string();
        let to = self.to.to_string();
        let mut capture = "".to_string();
        if let Some((captured, _)) = self.capturing {
            capture.push_str(&format!(" x{}", captured.as_char()));
        }

        // Non-standard representation, but fully detailed
        write!(f, "{} {}{}{}", piece, from, to, capture)
    }
}

impl PartialOrd for Ply {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ply {
    fn cmp(&self, other: &Self) -> Ordering {
        // PV first
        match (self.pv_move, other.pv_move) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            _ => (),
        }

        // using MVV_LVA (Most Valuable Victim, Least Valuable Attacker)
        match (self.capturing, other.capturing) {
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => self.moving_piece.0.cmp(&other.moving_piece.0),
            _ => self
                .capture_sorting_value()
                .cmp(&other.capture_sorting_value()),
        }
    }
}

impl Ply {
    fn capture_sorting_value(&self) -> u8 {
        if let Some(captured) = self.capturing {
            let victim_value = match captured.0.0 {
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
            victim_value + attacker_value
        } else {
            0
        }
    }
}

impl Bitboard {
    /// Returns a iterator of all unblocked single-step plys
    ///
    /// # Safety
    /// Will require `bitboard_ptr` to be valid until all movement generation has been done.
    /// The pointer needs to be the Bitboards array of Bitboards
    pub unsafe fn single_step_plys_in_dirs(
        &self,
        dirs: &[fn(&Self) -> Self],
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        by_piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        dirs.iter()
            .map(|dir| dir(self))
            .filter(|board| **board != 0 && **board & **blocked == 0)
            .map(move |board| {
                let mut capturing = None;
                if *board & **capturable != 0 {
                    // There is a capture present
                    let capturing_iter = unsafe {
                        all_pieces_by_color_from_ptr_iter(bitboard_ptr, by_piece.1.next())
                    };
                    for PieceWithBitboard(piece, opposing_board) in capturing_iter {
                        let capture = board & opposing_board;
                        if *capture != 0 {
                            capturing = Some((piece, capture.as_bit_idx()))
                        }
                    }
                }

                Ply {
                    moving_piece: by_piece,
                    from: self.as_bit_idx(),
                    to: board.as_bit_idx(),
                    capturing,
                    ..Default::default()
                }
            })
    }

    /// Returns a iterator of all unblocked multi-step plys (sliding pieces)
    ///
    /// # Safety
    /// Will require `bitboard_ptr` to be valid until all movement generation has been done.
    /// The pointer needs to be the Bitboards array of Bitboards
    pub unsafe fn multi_step_plys_in_dirs(
        &self,
        dirs: &[fn(&Self, &Self, &Self) -> Vec<Self>],
        blocked: &Self,
        capturable: &Self,
        bitboard_ptr: *const Bitboard,
        by_piece: Piece,
    ) -> impl Iterator<Item = Ply> {
        dirs.iter()
            .flat_map(|dir| dir(self, blocked, capturable))
            .map(move |board| {
                let mut capturing = None;
                if *board & **capturable != 0 {
                    // There is a capture present
                    let capturing_iter = unsafe {
                        all_pieces_by_color_from_ptr_iter(bitboard_ptr, by_piece.1.next())
                    };
                    for PieceWithBitboard(piece, opposing_board) in capturing_iter {
                        let capture = board & opposing_board;
                        if *capture != 0 {
                            capturing = Some((piece, capture.as_bit_idx()))
                        }
                    }
                }

                Ply {
                    moving_piece: by_piece,
                    from: self.as_bit_idx(),
                    to: board.as_bit_idx(),
                    capturing,
                    ..Default::default()
                }
            })
    }
}

impl Bitboards {
    pub fn make_ply(&mut self, ply: &Ply) {
        // Updating moving piece
        let moving_piece_idx = bitboard_idx(ply.moving_piece);
        self.boards[moving_piece_idx].set(ply.from, false);
        self.boards[moving_piece_idx].set(ply.to, true);

        // Update piece list
        for piece in self.piece_list[moving_piece_idx].iter_mut() {
            if piece == &ply.from {
                *piece = ply.to
            }
        }

        // Handle capturing
        if let Some((captured_piece, idx)) = ply.capturing {
            // update position boards
            let capturing_idx = bitboard_idx(captured_piece);
            self.boards[capturing_idx].set(idx, false);

            // update piece list
            for i in 0..self.piece_list[capturing_idx].len() {
                if self.piece_list[capturing_idx][i] == idx {
                    self.piece_list[capturing_idx].remove(i);
                    break;
                }
            }
        }

        // Handle linked move
        if let Some((other_piece, from, to)) = ply.also_move {
            let moving_piece_idx = bitboard_idx(other_piece);
            self.boards[moving_piece_idx].set(from, false);
            self.boards[moving_piece_idx].set(to, true);
        }

        // en passant
        let en_passant = ply.en_passant_board.unwrap_or(Bitboard(u256::ZERO));
        self.en_passant = en_passant;

        // update hash
        self.zobrist_hash = self
            .zobrist_table
            .update_hash_bitboard(self.zobrist_hash, ply);

        // update visited positions
        let mut check_cache = false;
        self.visited_positions
            .lock()
            .unwrap()
            .entry(*self.zobrist_hash)
            .and_modify(|i| {
                // Position already visted, checking cache
                check_cache = true;
                *i += 1
            })
            .or_insert(1);
        self.check_quiescence_table = check_cache;
    }

    pub fn unmake_ply(&mut self, ply: &Ply, previous_ply: Option<&Ply>) {
        // Updating moving piece
        let moving_piece_idx = bitboard_idx(ply.moving_piece);
        self.boards[moving_piece_idx].set(ply.to, false);
        self.boards[moving_piece_idx].set(ply.from, true);

        // Update piece list
        for piece in self.piece_list[moving_piece_idx].iter_mut() {
            if piece == &ply.to {
                *piece = ply.from
            }
        }

        // Handle capturing
        if let Some((captured_piece, idx)) = ply.capturing {
            // update position boards
            let capturing_idx = bitboard_idx(captured_piece);
            self.boards[capturing_idx].set(idx, true);

            // update piece list
            self.piece_list[capturing_idx].push(idx);
        }

        // Handle linked move
        if let Some((other_piece, from, to)) = ply.also_move {
            let moving_piece_idx = bitboard_idx(other_piece);
            self.boards[moving_piece_idx].set(to, false);
            self.boards[moving_piece_idx].set(from, true);
        }

        // restore en_passant
        if let Some(ply) = previous_ply {
            let en_passant = ply.en_passant_board.unwrap_or(Bitboard(u256::ZERO));
            self.en_passant = en_passant;
        } else {
            self.en_passant = Bitboard(u256::ZERO);
        }

        // update visited positions
        self.visited_positions
            .lock()
            .unwrap()
            .entry(*self.zobrist_hash)
            .and_modify(|i| *i -= 1);

        // returning to a previous position, so we can check cache
        self.check_quiescence_table = true;

        // update hash
        self.zobrist_hash = self
            .zobrist_table
            .update_hash_bitboard(self.zobrist_hash, ply);
    }

    fn legality_check(&self, last_move_by: PieceColor) -> bool {
        // thricefold repetiton check
        let thricefold_repetition = self
            .visited_positions
            .lock()
            .unwrap()
            .get(&self.zobrist_hash)
            .is_some_and(|i| *i >= 3);

        if thricefold_repetition {
            return false;
        }

        // king check
        let king_mask = self.boards[bitboard_idx(Piece(PieceType::King, last_move_by))];
        let opponent_en_prise = self.en_prise_by_color(last_move_by.next());

        *king_mask & *opponent_en_prise == 0
    }
}

pub fn legality_filter(
    iter: impl Iterator<Item = Ply>,
    boards: &mut Bitboards,
) -> impl Iterator<Item = Ply> {
    iter.filter(move |ply| {
        boards.make_ply(ply);
        let res = boards.legality_check(ply.moving_piece.1);
        boards.unmake_ply(ply, None);
        res
    })
}

pub fn captures_only(iter: impl Iterator<Item = Ply>) -> impl Iterator<Item = Ply> {
    iter.filter(|ply| ply.capturing.is_some())
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use ethnum::u256;

    use crate::chess_engine::{
        bitboard::{
            Bitboard, Bitboards, bitboard_idx,
            move_gen::{king::KING_DIRS, queen::QUEEN_STEP_DIRS},
        },
        pieces::*,
        zobrist::CHANGE_PLAYER_INDEX,
    };

    use super::Ply;

    #[test]
    fn single_step_plys() {
        let boards = Bitboards::new_from_str(
            r#"
            k0
            0P
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_KING)];

        let mut plys = unsafe {
            board
                .single_step_plys_in_dirs(
                    &KING_DIRS,
                    &boards.blocked_mask_for_color(PieceColor::White),
                    &boards.all_pieces_by_color(PieceColor::Black),
                    boards.boards.as_ptr(),
                    WHITE_KING,
                )
                .collect::<BinaryHeap<Ply>>()
        };

        assert_eq!(plys.len(), 3);
        assert!(plys.pop().unwrap().capturing.is_some())
    }

    #[test]
    fn multi_step_plys() {
        let boards = Bitboards::new_from_str(
            r#"
            q0P
            000
            000
            "#,
        );
        let board = boards.boards[bitboard_idx(WHITE_QUEEN)];

        let mut plys = unsafe {
            board
                .multi_step_plys_in_dirs(
                    &QUEEN_STEP_DIRS,
                    &boards.blocked_mask_for_color(PieceColor::White),
                    &boards.all_pieces_by_color(PieceColor::Black),
                    boards.boards.as_ptr(),
                    WHITE_QUEEN,
                )
                .collect::<BinaryHeap<Ply>>()
        };

        assert_eq!(plys.len(), 6);
        assert!(plys.pop().unwrap().capturing.is_some())
    }

    #[test]
    fn mvv_lva() {
        let pawn_takes_pawn = Ply {
            moving_piece: WHITE_PAWN,
            capturing: Some((BLACK_PAWN, 0.into())),
            ..Default::default()
        };

        let pawn_takes_queen = Ply {
            moving_piece: WHITE_PAWN,
            capturing: Some((BLACK_QUEEN, 0.into())),
            ..Default::default()
        };

        let queen_takes_pawn = Ply {
            moving_piece: WHITE_QUEEN,
            capturing: Some((BLACK_PAWN, 0.into())),
            ..Default::default()
        };

        let queen_takes_queen = Ply {
            moving_piece: WHITE_QUEEN,
            capturing: Some((BLACK_QUEEN, 0.into())),
            ..Default::default()
        };

        let queen_no_take = Ply {
            moving_piece: WHITE_QUEEN,
            ..Default::default()
        };

        let pawn_no_take = Ply {
            moving_piece: WHITE_PAWN,
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
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let mut expected = Bitboards::new_from_str(
            r#"
        p
        0
        "#,
        );
        expected.zobrist_hash ^= expected.zobrist_table.table[CHANGE_PLAYER_INDEX];

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(bitboard.zobrist_hash, expected.zobrist_hash);
    }

    #[test]
    fn unmake_ply() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let expected = bitboard.clone();

        let ply = Ply {
            moving_piece: WHITE_PAWN,
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
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0P
        p0
        "#,
        );

        let mut expected = Bitboards::new_from_str(
            r#"
        0p
        00
        "#,
        );
        expected.zobrist_hash ^= expected.zobrist_table.table[CHANGE_PLAYER_INDEX];

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 1.into(),
            capturing: Some((BLACK_PAWN, 1.into())),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(bitboard, expected);
    }

    #[test]
    fn unmake_capture_ply() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let expected = bitboard.clone();

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 1.into(),
            capturing: Some((BLACK_PAWN, 1.into())),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        bitboard.unmake_ply(&ply, None);
        assert_eq!(bitboard, expected);
    }

    #[test]
    fn make_en_passant_ply() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        00
        00
        p0
        "#,
        );

        let expected = Bitboards::new_from_str(
            r#"
        00
        p0
        00
        "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_PAWN)];
        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 32.into(),
            to: 0.into(),
            en_passant_board: Some(Bitboard(u256::ONE << 16)),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(bitboard.en_passant, expected);
    }

    #[test]
    fn unmake_en_passant_ply() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        00
        00
        p0
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 32.into(),
            to: 0.into(),
            en_passant_board: Some(Bitboard(u256::ONE << 16)),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        bitboard.unmake_ply(&ply, None);
        assert_eq!(bitboard.en_passant, Bitboard(u256::ZERO));
    }

    #[test]
    fn unmake_en_passant_restore_ply() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        00
        00
        p0
        "#,
        );

        let expected = Bitboards::new_from_str(
            r#"
        00
        p0
        00
        "#,
        );
        let expected = expected.boards[bitboard_idx(WHITE_PAWN)];

        let first_ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 32.into(),
            to: 0.into(),
            en_passant_board: Some(Bitboard(u256::ONE << 16)),
            ..Default::default()
        };

        let second_ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 0.into(),
            to: 16.into(),
            ..Default::default()
        };

        bitboard.make_ply(&first_ply);
        bitboard.make_ply(&second_ply);
        bitboard.unmake_ply(&second_ply, Some(&first_ply));
        assert_eq!(bitboard.en_passant, expected);
    }

    #[test]
    fn make_ply_visited_count() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert_eq!(
            bitboard
                .visited_positions
                .lock()
                .unwrap()
                .get(&bitboard.zobrist_hash),
            Some(&1)
        );
    }

    #[test]
    fn unmake_ply_visited_count() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        let hash = bitboard.zobrist_hash;
        bitboard.unmake_ply(&ply, None);

        assert_eq!(
            bitboard.visited_positions.lock().unwrap().get(&hash),
            Some(&0)
        );
    }

    #[test]
    fn legal_move() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        R0
        r0
        k0
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_ROOK,
            from: 16.into(),
            to: 0.into(),
            capturing: Some((BLACK_ROOK, 0.into())),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert!(bitboard.legality_check(ply.moving_piece.1));
    }

    #[test]
    fn illegal_move() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        R0
        r0
        k0
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_ROOK,
            from: 16.into(),
            to: 17.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        assert!(!bitboard.legality_check(ply.moving_piece.1));
    }

    #[test]
    fn make_ply_update_piece_list() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        let bitboard_idx = bitboard_idx(WHITE_PAWN);
        assert_eq!(bitboard.piece_list[bitboard_idx], vec![0.into()]);
    }

    #[test]
    fn unmake_ply_update_piece_list() {
        let mut bitboard = Bitboards::new_from_str(
            r#"
        0
        p
        "#,
        );

        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        bitboard.make_ply(&ply);
        bitboard.unmake_ply(&ply, None);
        let bitboard_idx = bitboard_idx(WHITE_PAWN);
        assert_eq!(bitboard.piece_list[bitboard_idx], vec![16.into()]);
    }

    #[test]
    fn display_ply() {
        let ply = Ply {
            moving_piece: WHITE_PAWN,
            from: 16.into(),
            to: 0.into(),
            ..Default::default()
        };

        assert_eq!(ply.to_string().as_str(), "p A2A1");
    }

    #[test]
    fn display_capturing_ply() {
        let ply = Ply {
            moving_piece: BLACK_ROOK,
            from: 31.into(),
            to: 16.into(),
            capturing: Some((WHITE_QUEEN, 0.into())),
            ..Default::default()
        };

        assert_eq!(ply.to_string().as_str(), "R P2A2 xq");
    }
}
