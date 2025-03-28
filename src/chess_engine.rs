use std::{
    collections::HashMap,
    fmt::Display,
    i32,
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::prelude::*;

pub struct ChessEnginePlugin;
impl Plugin for ChessEnginePlugin {
    fn build(&self, app: &mut App) {
        let mut board = Board::default();

        loop {
            println!("{}", board);
            if let Some(next_move) = board.search_next_move(3).1 {
                board.apply_move(next_move);
                std::thread::sleep(Duration::from_secs_f32(1.0));
            } else {
                break;
            }
        }
        app.init_resource::<Board>();
    }
}

#[derive(Resource, Debug, Clone)]
struct Board {
    board: Vec<Option<Piece>>,
    row_length: usize,
    moves: Vec<Move>,
    king_map: HashMap<PieceColor, Pos>,
    next_move_by: PieceColor,
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
    fn from_str(input: &str, row_length: usize) -> Self {
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
                'p' | 'P' => PieceType::Pawn(true),
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

    fn get_moves_for_pos(&self, pos: Pos) -> Option<Vec<Move>> {
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
    fn row_and_column_to_idx(&self, row: usize, column: usize) -> usize {
        row * self.row_length + column
    }

    #[inline]
    fn pos_to_idx(&self, pos: Pos) -> usize {
        self.row_and_column_to_idx(pos.row, pos.column)
    }

    /// Returns None if new pos wouldn't be valid
    fn apply_vec_to_pos(&self, pos: Pos, movement_vec: &MoveVec) -> Option<Pos> {
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
    fn legality_check(&self, moves: Vec<Move>) -> Vec<Move> {
        // if there are no moves, return early
        if moves.is_empty() {
            return moves;
        }

        // if there is no king (i.e. simulations, tests), ignore legality checks
        if self.king_map.get(&moves[0].by).is_none() {
            return moves;
        }

        let mut legal_moves = vec![];
        let mut sim_board = self.clone();
        'next: for this_move in moves {
            sim_board.apply_move(this_move);

            for tile in sim_board.board.iter() {
                if let Some(piece) = tile {
                    // We only care about the other color
                    if piece.color != this_move.by {
                        // We don't really care if the opponents move is legal, just if the king is threatened
                        let moves = piece.generate_pseudolegal_moves(&sim_board);

                        // if any of the pseudolegal moves contain the king's position, it is threatened
                        if moves.iter().any(|next_move| {
                            next_move.move_to.to == *sim_board.king_map.get(&this_move.by).unwrap()
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

    fn apply_move(&mut self, this_move: Move) {
        // push to simulation list
        self.moves.push(this_move);

        // capture
        if let Some(captured_piece) = this_move.capturing {
            self[captured_piece.pos] = None;
        }

        // update inner pos on piece
        let mut piece = self[this_move.move_to.from].unwrap();
        piece.pos = this_move.move_to.to;

        // if piece is king, we also need to move it's mapping
        if piece.piece_type == PieceType::King {
            self.king_map.insert(piece.color, this_move.move_to.to);
        }

        // move piece
        self[this_move.move_to.from] = None;
        self[this_move.move_to.to] = Some(piece);
        self.next_move_by = self.next_move_by.next();
    }

    fn rewind_last_move(&mut self) {
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
        const ISOLATED_PAWN_WEIGHT: i32 = 1;
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
                if matches!(piece.piece_type, PieceType::Pawn(_)) {
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
                PieceType::Pawn(_) => material_score += count * PAWN_WEIGHT,
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

    fn unsorted_move_list(&self) -> Vec<Move> {
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

    fn search_next_move(&self, depth: i8) -> (i32, Option<Move>) {
        self.alpha_beta(i32::MIN, i32::MAX, depth)
    }

    fn alpha_beta(&self, mut alpha: i32, beta: i32, depth: i8) -> (i32, Option<Move>) {
        if depth == 0 {
            return (self.evaluate(), self.moves.last().cloned());
        };

        let mut sim_board = self.clone();

        let mut best_move = (i32::MIN, None);
        for this_move in self.unsorted_move_list() {
            sim_board.apply_move(this_move);
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

/// Valid position on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    row: usize,
    column: usize,
}
impl Pos {
    fn new(row: usize, column: usize) -> Self {
        Pos { row, column }
    }

    fn move_to(&self, other: &Pos) -> MoveTo {
        MoveTo {
            from: *self,
            to: *other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MoveTo {
    from: Pos,
    to: Pos,
}

/// Used with a Pos to generate a potentially new valid Pos on the board
#[derive(Debug, Clone, Copy)]
struct MoveVec {
    x: i16,
    y: i16,
}

/// Type that encodes the `MoveTo` with metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Move {
    move_to: MoveTo,
    by: PieceColor,
    /// Set if a move takes an opponent's piece
    capturing: Option<Piece>,
    /// Set if this move allows a pawn to be attacked via en passant
    en_passant_flag: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct Piece {
    piece_type: PieceType,
    color: PieceColor,
    pos: Pos,
}
impl Piece {
    fn new(piece_type: PieceType, color: PieceColor, pos: Pos) -> Self {
        Self {
            piece_type,
            color,
            pos,
        }
    }

    fn to_char(&self) -> char {
        match (self.piece_type, self.color) {
            (PieceType::King, PieceColor::White) => 'k',
            (PieceType::King, PieceColor::Black) => 'K',
            (PieceType::Queen, PieceColor::White) => 'q',
            (PieceType::Queen, PieceColor::Black) => 'Q',
            (PieceType::Rook, PieceColor::White) => 'r',
            (PieceType::Rook, PieceColor::Black) => 'R',
            (PieceType::Bishop, PieceColor::White) => 'b',
            (PieceType::Bishop, PieceColor::Black) => 'B',
            (PieceType::Knight, PieceColor::White) => 'n',
            (PieceType::Knight, PieceColor::Black) => 'N',
            (PieceType::Pawn(_), PieceColor::White) => 'p',
            (PieceType::Pawn(_), PieceColor::Black) => 'P',
        }
    }

    /// The generated moves do not perform any checking checks, however vector attacks do stop at collisions
    fn generate_pseudolegal_moves(&self, board: &Board) -> Vec<Move> {
        match self.piece_type {
            PieceType::King => self.king_move_generation(board),
            PieceType::Queen => self.queen_move_generation(board),
            PieceType::Rook => self.rook_move_generation(board),
            PieceType::Bishop => self.bishop_move_generation(board),
            PieceType::Knight => self.knight_move_generation(board),
            PieceType::Pawn(_) => self.pawn_move_generation(board),
        }
    }

    fn king_move_generation(&self, board: &Board) -> Vec<Move> {
        let movement_vectors = [
            MoveVec { x: 1, y: 0 },
            MoveVec { x: 1, y: 1 },
            MoveVec { x: 0, y: 1 },
            MoveVec { x: -1, y: 1 },
            MoveVec { x: -1, y: 0 },
            MoveVec { x: -1, y: -1 },
            MoveVec { x: 0, y: -1 },
            MoveVec { x: 1, y: -1 },
        ];

        self.step_moves(board, &movement_vectors)
    }

    fn knight_move_generation(&self, board: &Board) -> Vec<Move> {
        let movement_vectors = [
            MoveVec { x: 1, y: 2 },
            MoveVec { x: 2, y: 1 },
            MoveVec { x: -1, y: 2 },
            MoveVec { x: 2, y: -1 },
            MoveVec { x: -2, y: 1 },
            MoveVec { x: 1, y: -2 },
            MoveVec { x: -1, y: -2 },
            MoveVec { x: -2, y: -1 },
        ];

        self.step_moves(board, &movement_vectors)
    }

    fn step_moves(&self, board: &Board, movement_vectors: &[MoveVec]) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        for vec in movement_vectors {
            // `valid_dest` is definitely in range of the board
            if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, vec) {
                // perform `Tile::Inactive` and destination piece color checks
                let tile = board[valid_dest];
                match tile {
                    None => moves.push(self.move_to_pos(valid_dest)),
                    Some(other_piece) if self.color != other_piece.color => {
                        moves.push(self.move_capture(&other_piece))
                    }
                    _ => (),
                }
            }
        }

        moves
    }

    fn raycasted_moves(&self, board: &Board, movement_vectors: &[MoveVec]) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        for vec in movement_vectors {
            self.vector_walk(board, &mut moves, vec);
        }

        moves
    }

    fn rook_move_generation(&self, board: &Board) -> Vec<Move> {
        let movement_vectors = [
            MoveVec { x: 1, y: 0 },
            MoveVec { x: 0, y: 1 },
            MoveVec { x: -1, y: 0 },
            MoveVec { x: 0, y: -1 },
        ];

        self.raycasted_moves(board, &movement_vectors)
    }

    fn bishop_move_generation(&self, board: &Board) -> Vec<Move> {
        let movement_vectors = [
            MoveVec { x: 1, y: 1 },
            MoveVec { x: -1, y: 1 },
            MoveVec { x: -1, y: -1 },
            MoveVec { x: 1, y: -1 },
        ];
        self.raycasted_moves(board, &movement_vectors)
    }

    fn queen_move_generation(&self, board: &Board) -> Vec<Move> {
        let movement_vectors = [
            MoveVec { x: 1, y: 0 },
            MoveVec { x: 1, y: 1 },
            MoveVec { x: 0, y: 1 },
            MoveVec { x: -1, y: 1 },
            MoveVec { x: -1, y: 0 },
            MoveVec { x: -1, y: -1 },
            MoveVec { x: 0, y: -1 },
            MoveVec { x: 1, y: -1 },
        ];

        self.raycasted_moves(board, &movement_vectors)
    }

    fn pawn_move_generation(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];

        let direction = if self.color == PieceColor::White {
            -1
        } else {
            1
        };
        let move_vec = MoveVec {
            x: 1 * direction,
            y: 0,
        };

        // forward move
        if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &move_vec) {
            // Checking that the pawn isn't blocked
            if board[valid_dest].is_none() {
                moves.push(self.move_to_pos(valid_dest));

                // If the first step isn't blocked, we can check for the double step
                if self.piece_type == PieceType::Pawn(true) {
                    let move_vec = MoveVec {
                        x: 2 * direction,
                        y: 0,
                    };
                    if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &move_vec) {
                        // Checking that the pawn isn't blocked
                        if board[valid_dest].is_none() {
                            moves.push(self.move_to_pos_en_passant(valid_dest));
                        }
                    }
                }
            }
        }

        // diagonal taking
        let move_vecs = [
            MoveVec {
                x: 1 * direction,
                y: -1,
            },
            MoveVec {
                x: 1 * direction,
                y: 1,
            },
        ];
        for vec in move_vecs {
            if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &vec) {
                let other_piece = board[valid_dest];
                if let Some(other_piece) = other_piece {
                    if other_piece.color != self.color {
                        moves.push(self.move_capture(&other_piece));
                    }
                }
            }
        }

        // en passant
        let last_move = board.moves.last().cloned();

        if let Some(last_move) = last_move {
            if last_move.en_passant_flag
                && last_move.move_to.to.row == self.pos.row
                && last_move.move_to.to.column.abs_diff(self.pos.column) == 1
            {
                let move_vec = MoveVec {
                    x: direction,
                    y: last_move.move_to.to.column as i16 - self.pos.column as i16,
                };
                if let Some(valid_dest) = board.apply_vec_to_pos(self.pos, &move_vec) {
                    if let Some(other_piece) = board[last_move.move_to.to] {
                        if other_piece.color != self.color {
                            moves.push(self.move_to_while_capturing(valid_dest, &other_piece));
                        }
                    }
                }
            }
        }

        moves
    }
    /// walk into the direction of a MoveVec until we reach and `Tile::Inactive`, a piece of our color,
    /// or we pass a piece of the opponent's color.
    fn vector_walk(&self, board: &Board, moves: &mut Vec<Move>, vec: &MoveVec) {
        let mut finished = false;
        let mut pos = self.pos;
        while !finished {
            if let Some(valid_dest) = board.apply_vec_to_pos(pos, vec) {
                // perform out of bound and destination piece color checks
                let tile = board[valid_dest];
                match tile {
                    None => {
                        moves.push(self.move_to_pos(valid_dest));
                        pos = valid_dest
                    }
                    Some(other_piece) if self.color != other_piece.color => {
                        moves.push(self.move_capture(&other_piece));
                        finished = true;
                    }
                    _ => finished = true,
                }
            } else {
                finished = true;
            }
        }
    }

    fn move_to_pos(&self, pos: Pos) -> Move {
        Move {
            move_to: self.pos.move_to(&pos),
            by: self.color,
            capturing: None,
            en_passant_flag: false,
        }
    }

    fn move_to_pos_en_passant(&self, pos: Pos) -> Move {
        Move {
            move_to: self.pos.move_to(&pos),
            by: self.color,
            capturing: None,
            en_passant_flag: true,
        }
    }

    fn move_capture(&self, piece: &Piece) -> Move {
        Move {
            move_to: self.pos.move_to(&piece.pos),
            by: self.color,
            capturing: Some(*piece),
            en_passant_flag: false,
        }
    }

    fn move_to_while_capturing(&self, pos: Pos, piece: &Piece) -> Move {
        Move {
            move_to: self.pos.move_to(&pos),
            by: self.color,
            capturing: Some(*piece),
            en_passant_flag: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    /// bool tracks if pawn can take two steps forwards
    Pawn(bool),
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PieceColor {
    White,
    Black,
}
impl PieceColor {
    fn score_sign(&self) -> i32 {
        match self {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        }
    }

    fn pawn_move_direction(&self) -> i32 {
        match self {
            PieceColor::White => -1,
            PieceColor::Black => 1,
        }
    }
    fn next(&self) -> Self {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn apply_vec_to_pos() {
        let board = Board::from_str(
            r#"
            000
            000
            000
            000
            "#,
            3,
        );

        let pos = Pos::new(1, 1);
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: 1, y: 1 })
                .is_some()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: -1, y: -1 })
                .is_some()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: 2, y: 1 })
                .is_some()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: 3, y: 1 })
                .is_none()
        );
        assert!(
            board
                .apply_vec_to_pos(pos, &MoveVec { x: -2, y: 1 })
                .is_none()
        );
    }

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

        assert_eq!(
            dict.get(&(PieceType::Pawn(true), PieceColor::White)),
            Some(&8)
        );
        assert_eq!(
            dict.get(&(PieceType::Pawn(true), PieceColor::Black)),
            Some(&8)
        );
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
    fn king_move_generation() {
        let board = Board::from_str(
            r#"
        0kP
        p00
        "#,
            3,
        );
        let pseudolegal_count = 4;

        let piece = board[Pos::new(0, 1)].unwrap();
        let enemy_pawn = board[Pos::new(0, 2)].unwrap();
        let friend_pawn = board[Pos::new(1, 0)].unwrap();

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), pseudolegal_count);
        assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
        assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    }

    #[test]
    fn rook_move_generation() {
        let board = Board::from_str(
            r#"
        prPP
        0000
        0000
        "#,
            4,
        );
        let pseudolegal_count = 3;

        let piece = board[Pos::new(0, 1)].unwrap();
        let enemy_pawn = board[Pos::new(0, 2)].unwrap();
        let obscured_pawn = board[Pos::new(0, 3)].unwrap();
        let friend_pawn = board[Pos::new(0, 0)].unwrap();
        let far_away_pos = Pos::new(2, 1);

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), pseudolegal_count);
        assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
        assert!(!moves.contains(&piece.move_capture(&obscured_pawn)));
        assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
        assert!(moves.contains(&piece.move_to_pos(far_away_pos)));
    }

    #[test]
    fn bishop_move_generation() {
        let board = Board::from_str(
            r#"
        0000P
        0p0P0
        00b00
        00000
        00000
        "#,
            5,
        );
        let pseudolegal_count = 5;

        let piece = board[Pos::new(2, 2)].unwrap();
        let enemy_pawn = board[Pos::new(1, 3)].unwrap();
        let obscured_pawn = board[Pos::new(0, 4)].unwrap();
        let friend_pawn = board[Pos::new(1, 1)].unwrap();
        let far_away_pos = Pos::new(4, 0);

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), pseudolegal_count);
        assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
        assert!(!moves.contains(&piece.move_capture(&obscured_pawn)));
        assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
        assert!(moves.contains(&piece.move_to_pos(far_away_pos)));
    }

    #[test]
    fn queen_move_generation() {
        let board = Board::from_str(
            r#"
        0000P
        0p0P0
        00q00
        00000
        00RN0
        "#,
            5,
        );
        let pseudolegal_count = 13;

        let piece = board[Pos::new(2, 2)].unwrap();
        let enemy_pawn = board[Pos::new(1, 3)].unwrap();
        let obscured_pawn = board[Pos::new(0, 4)].unwrap();
        let friend_pawn = board[Pos::new(1, 1)].unwrap();
        let far_away_rook = board[Pos::new(4, 2)].unwrap();
        let untargeted_knight = board[Pos::new(4, 3)].unwrap();

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), pseudolegal_count);
        assert_eq!(moves.len(), pseudolegal_count);
        assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
        assert!(!moves.contains(&piece.move_capture(&obscured_pawn)));
        assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
        assert!(moves.contains(&piece.move_capture(&far_away_rook)));
        assert!(!moves.contains(&piece.move_capture(&untargeted_knight)));
    }

    #[test]
    fn knight_move_generation() {
        let board = Board::from_str(
            r#"
        000P0
        0000p
        00n00
        00000
        00000
        "#,
            5,
        );
        let pseudolegal_count = 7;

        let piece = board[Pos::new(2, 2)].unwrap();
        let enemy_pawn = board[Pos::new(0, 3)].unwrap();
        let friend_pawn = board[Pos::new(1, 4)].unwrap();

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), pseudolegal_count);
        assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
        assert!(!moves.contains(&piece.move_capture(&friend_pawn)));
    }

    #[test]
    fn pawn_move_generation() {
        let board = Board::from_str(
            r#"
        0000
        00P0
        0p00
        "#,
            4,
        );
        let pseudolegal_count = 3;

        let piece = board[Pos::new(2, 1)].unwrap();
        let enemy_pawn = board[Pos::new(1, 2)].unwrap();
        let normal_pos = Pos::new(1, 1);
        let double_step_pos = Pos::new(0, 1);

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), pseudolegal_count);
        assert!(moves.contains(&piece.move_capture(&enemy_pawn)));
        assert!(moves.contains(&piece.move_to_pos(normal_pos)));
        assert!(moves.contains(&piece.move_to_pos_en_passant(double_step_pos)));
    }

    #[test]
    fn keep_track_of_kings_test() {
        let board = Board::from_str(
            r#"
            0k
            K0
            "#,
            2,
        );

        assert_eq!(
            board.king_map.get(&PieceColor::White),
            Some(&Pos::new(0, 1))
        );
        assert_eq!(
            board.king_map.get(&PieceColor::Black),
            Some(&Pos::new(1, 0))
        );
    }

    #[test]
    fn pawn_en_passant_test() {
        let mut board = Board::from_str(
            r#"
            00
            00
            Pp
            "#,
            2,
        );
        board.moves.push(super::Move {
            move_to: MoveTo {
                from: Pos::new(0, 0),
                to: Pos::new(2, 0),
            },
            by: PieceColor::Black,
            capturing: None,
            en_passant_flag: true,
        });
        let valid_moves = 3;

        let piece = board[Pos::new(2, 1)].unwrap();
        let enemy_pawn = board[Pos::new(2, 0)].unwrap();
        let dest = Pos::new(1, 0);

        let moves = piece.generate_pseudolegal_moves(&board);
        assert_eq!(moves.len(), valid_moves);
        assert!(moves.contains(&piece.move_to_while_capturing(dest, &enemy_pawn)));
    }

    #[test]
    fn legal_moves_checking_test() {
        let board = Board::from_str(
            r#"
            00k0n
            r00b0
            00R00
            "#,
            5,
        );
        let legal_moves = 6;

        let moves = board.unsorted_move_list();

        assert_eq!(moves.len(), legal_moves);
    }

    #[test]
    fn kingless_legal_move_test() {
        let board = Board::from_str(
            r#"
            0000n
            r00b0
            00R00
            "#,
            5,
        );
        let legal_moves = 9;

        let moves = board.unsorted_move_list();

        assert_eq!(moves.len(), legal_moves);
    }

    #[test]
    fn apply_move() {
        let mut board = Board::from_str(
            r#"
            P0
            0p
            "#,
            2,
        );
        let dest = Pos::new(0, 0);
        let this_move = super::Move {
            move_to: MoveTo {
                from: Pos::new(1, 1),
                to: dest,
            },
            by: PieceColor::White,
            capturing: Some(Piece {
                piece_type: PieceType::Pawn(true),
                color: PieceColor::Black,
                pos: dest,
            }),
            en_passant_flag: false,
        };

        board.apply_move(this_move);
        assert_eq!(board.board.len(), 4);

        assert_eq!(board.moves.len(), 1);
        assert_eq!(
            board.board[0],
            Some(Piece {
                piece_type: PieceType::Pawn(true),
                color: PieceColor::White,
                pos: dest
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
        let this_move = super::Move {
            move_to: MoveTo {
                from: Pos::new(1, 1),
                to: Pos::new(0, 0),
            },
            by: PieceColor::White,
            capturing: Some(Piece {
                piece_type: PieceType::Pawn(true),
                color: PieceColor::Black,
                pos: Pos::new(0, 0),
            }),
            en_passant_flag: false,
        };

        board.apply_move(this_move);
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
            p0p0p
            p0p0p
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
        let expected_move = super::Move {
            move_to: MoveTo {
                from: Pos::new(1, 1),
                to: Pos::new(0, 0),
            },
            by: PieceColor::White,
            capturing: Some(Piece {
                piece_type: PieceType::Rook,
                color: PieceColor::Black,
                pos: Pos::new(0, 0),
            }),
            en_passant_flag: false,
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
        let expected_move = super::Move {
            move_to: MoveTo {
                from: Pos::new(0, 0),
                to: Pos::new(0, 4),
            },
            by: PieceColor::Black,
            capturing: Some(Piece {
                piece_type: PieceType::Rook,
                color: PieceColor::White,
                pos: Pos::new(0, 4),
            }),
            en_passant_flag: false,
        };

        assert!(next_move.1.is_some());
        assert_eq!(next_move.1, Some(expected_move));
    }
}
