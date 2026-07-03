use std::fmt;

use crate::board::FenParseError::*;

pub struct Board {
    // 12 64 bit int bitboards - 2 colors, 6 pieces
    piece_bitboards: [u64; 12], // [white pawn, black pawn, ..., white king, black king]

    // 3 occupancy bitboards - white occupancy, black occupancy, general occupancy (union of both)
    white_occu: u64,
    black_occu: u64,
    full_occu: u64,

    // active color
    active: bool, // white = true, black = false

    // castling rights
    white_castling: [bool; 2], // white - [can castle kingside, can castle queenside]
    black_castling: [bool; 2], // black - [can castle kingside, can castle queenside]

    // en passant target square     (if a pawn moved from e2 - e4, e3 was skipped an is en passant available)
    ep_target: i8, // 0-63 top left board corner to bottom right,
    // moving left to right, top to bottom
    // -1 means not applicable

    // half and full move clocks
    half_clock: u8, // number of half moves since last pawn advance or piece capture
    full_clock: u8, // number of full moves since start of game
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Piece {
    WhitePawn,
    BlackPawn,
    WhiteKnight,
    BlackKnight,
    WhiteBishop,
    BlackBishop,
    WhiteRook,
    BlackRook,
    WhiteQueen,
    BlackQueen,
    WhiteKing,
    BlackKing,
}

#[derive(Debug)]
pub enum FenParseError {
    InvalidCodeLengthError(String),
    InvalidPieceError(u8),
}

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenParseError::InvalidCodeLengthError(code) => write!(
                f,
                "InvalidCodeLengthError: '{}' does not contain the expected number of fields.",
                code
            ),
            FenParseError::InvalidPieceError(byte) => write!(
                f,
                "InvalidPieceError: '{}' is not a valid Piece type.",
                char::from(*byte)
            ),
        }
    }
}

impl std::error::Error for FenParseError {}

impl TryFrom<u8> for Piece {
    type Error = FenParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'P' => Ok(Piece::WhitePawn),
            b'p' => Ok(Piece::BlackPawn),
            b'N' => Ok(Piece::WhiteKnight),
            b'n' => Ok(Piece::BlackKnight),
            b'B' => Ok(Piece::WhiteBishop),
            b'b' => Ok(Piece::BlackBishop),
            b'R' => Ok(Piece::WhiteRook),
            b'r' => Ok(Piece::BlackRook),
            b'Q' => Ok(Piece::WhiteQueen),
            b'q' => Ok(Piece::BlackQueen),
            b'K' => Ok(Piece::WhiteKing),
            b'k' => Ok(Piece::BlackKing),
            _ => Err(InvalidPieceError(value)),
        }
    }
}

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Board {
    pub fn new() -> Result<Self, FenParseError> {
        Self::from_fen(DEFAULT_FEN)
    }

    pub fn from_fen(fen_code: &str) -> Result<Self, FenParseError> {
        // build board from FEN notation
        let mut piece_bitboards: [u64; 12] = [0x0; 12];
        let mut white_occu: u64 = 0x0;
        let mut black_occu: u64 = 0x0;
        let mut full_occu: u64 = 0x0;
        let mut active = true;
        let mut white_castling: [bool; 2] = [true; 2];
        let mut black_castling: [bool; 2] = [true; 2];
        let mut ep_target: i8 = -1; // default to no applicable en passant
        let mut half_clock: u8 = 0;
        let mut full_clock: u8 = 1;

        let mut square_index: i8 = -1;
        let def_bitstring = 0x1; // for shifting

        let fen_code_parts: Vec<&str> = fen_code.split_whitespace().collect();
        if fen_code_parts.len() != 6 {
            return Err(InvalidCodeLengthError(String::from(fen_code)));
        }

        for byte in fen_code.bytes() {
            match byte {
                b'/' => continue,
                b'1'..=b'8' => {
                    let empty_squares = (byte - b'0') as i8;
                    square_index += empty_squares;
                }
                _ => {
                    square_index += 1;

                    let change_bitstring: u64 = def_bitstring << (63 - square_index);

                    let piece = Piece::try_from(byte)?;

                    piece_bitboards[piece as usize] |= change_bitstring;
                }
            }
        }

        Ok(Board {
            piece_bitboards: piece_bitboards,
            white_occu: white_occu,
            black_occu: black_occu,
            full_occu: full_occu,
            active: active,
            white_castling: white_castling,
            black_castling: black_castling,
            ep_target: ep_target,
            half_clock: half_clock,
            full_clock: full_clock,
        })
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
