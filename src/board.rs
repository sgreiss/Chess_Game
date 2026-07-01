pub struct Board {
    // 12 64bit int bitboards - 2 colors, 6 pieces
    // 3 occupancy bitboards - white occupancy, black occupancy, general occupancy (union of both)
    // active color
    // castling rights
    // en passant target square
    // half and full move clocks
}

impl Board {
    pub fn new() -> Self {
        // build default game start board
        Board {}
    }

    pub fn from_fen() -> Self {
        // build board from FEN notation
        Board {}
    }
}

/*
    ex. FEN notation for reference:

        rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

    1. Piece Placement: Board rows (ranks) from top to bottom, separated by slashes /. Numbers represent empty squares, and letters identify pieces (Uppercase = White, Lowercase = Black).
    2. Active Color: w (White to move) or b (Black to move).
    3. Castling Rights: KQkq (White can castle kingside/queenside, Black same). - if no one can castle.
    4. En Passant Target Square: The square an en passant pawn could capture into. - if none.
    5. Halfmove Clock: Number of half-moves since the last pawn advance or piece capture (for the 50-move rule).
    6. Fullmove Number: The number of the full move (starts at 1, incremented after Black's move).
*/
