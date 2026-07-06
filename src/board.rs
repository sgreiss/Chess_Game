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
    white_active: bool, // white = true, black = false

    // castling rights
    white_castling: [bool; 2], // white - [can castle kingside, can castle queenside]
    black_castling: [bool; 2], // black - [can castle kingside, can castle queenside]

    // en passant target square     (if a pawn moved from e2 - e4, e3 was skipped an is en passant available)
    ep_target: i8, // 0-63 top left board corner to bottom right,
    // moving left to right, top to bottom
    // -1 means not applicable

    // half and full move clocks
    half_clock: u8, // number of half moves since last pawn advance or piece capture, valid (0, 100) inclusive
    full_clock: u16, // number of full moves since start of game, valid 1+
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
    InvalidFieldCount(String),
    InvalidFieldLengthError(String, u8),
    InvalidPieceError(u8),
    InvalidStartingColorError(u8),
    InvalidCastlingArgumentError(u8),
    InvalidEPArgumentError(String),
    HalfClockArgumentNotInRangeError(u8),
    InvalidHalfClockArgumentError(String),
    FullClockArgumentNotInRangeError(u16),
    InvalidFullClockArgumentError(String),
}

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenParseError::InvalidFieldCount(code) => write!(
                f,
                "InvalidFieldCount: '{}' does not contain the expected number of fields: [placement][active][castling][en passant][half clock][full clock]",
                code
            ),
            FenParseError::InvalidFieldLengthError(code_splice, field) => {
                let field_name = match field {
                    0 => "placement",
                    1 => "active",
                    2 => "castling",
                    3 => "en passant",
                    4 => "half clock",
                    5 => "full clock",
                    _ => "",
                };
                write!(
                    f,
                    "InvalidFieldLengthError: '{}' is not the proper length for the field [{}]",
                    code_splice, field_name
                )
            }
            FenParseError::InvalidPieceError(piece_byte) => write!(
                f,
                "InvalidPieceError: '{}' is not a valid Piece type.",
                char::from(*piece_byte)
            ),
            FenParseError::InvalidStartingColorError(color_byte) => write!(
                f,
                "InvalidStartingColorError: '{}' is not a valid color type.",
                char::from(*color_byte)
            ),
            FenParseError::InvalidCastlingArgumentError(castling_byte) => write!(
                f,
                "InvalidCastlingArgumentError: '{}' is not a valid castling argument.",
                char::from(*castling_byte)
            ),
            FenParseError::InvalidEPArgumentError(ep_code_splice) => write!(
                f,
                "InvalidEnPassantArgumentError: '{}' is not a recognized target square or '-'.",
                ep_code_splice
            ),
            FenParseError::HalfClockArgumentNotInRangeError(clock) => write!(
                f,
                "HalfClockArgumentNotInRangeError: '{}' does not fall within the allowed range for the half clock field: (0, 100) inclusive.",
                clock
            ),
            FenParseError::InvalidHalfClockArgumentError(half_clock_splice) => write!(
                f,
                "InvalidHalfClockArgumentError: '{}' must be an integer in the inclusive range (0, 100).",
                half_clock_splice
            ),
            FenParseError::FullClockArgumentNotInRangeError(clock) => write!(
                f,
                "FullClockArgumentNotInRangeError: '{}' does not fall within the allowed range for the full clock field of positve integers.",
                clock
            ),
            FenParseError::InvalidFullClockArgumentError(full_clock_splice) => write!(
                f,
                "InvalidFullClockArgumentError: '{}' must be a postive integer greater than or equal to '1'.",
                full_clock_splice
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
        let mut white_active = true;
        let mut white_castling: [bool; 2] = [false; 2];
        let mut black_castling: [bool; 2] = [false; 2];
        let mut ep_target: i8 = -1; // default to no applicable en passant
        let half_clock: u8;
        let full_clock: u16;

        let mut square_index: i8 = -1;
        let def_bitstring = 0x1; // for shifting

        let fen_code_parts: Vec<&str> = fen_code.split_whitespace().collect();
        if fen_code_parts.len() != 6 {
            return Err(InvalidFieldCount(String::from(fen_code)));
        }

        // 1. Piece Placement

        let mut slash_count = 0;

        for byte in fen_code_parts[0].bytes() {
            // read over the Piece Placement section of the FEN code
            match byte {
                b'/' => {
                    slash_count += 1;
                    continue;
                }
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

        if slash_count != 7 {
            return Err(InvalidFieldLengthError(String::from(fen_code_parts[0]), 0));
        }

        // 2. Active Color

        for byte in fen_code_parts[1].bytes() {
            match byte {
                b'w' => white_active = true,
                b'b' => white_active = false,
                _ => return Err(InvalidStartingColorError(byte)),
            }
        }

        // 3. Castling Rights

        for byte in fen_code_parts[2].bytes() {
            match byte {
                b'K' => white_castling[0] = true,
                b'Q' => white_castling[1] = true,
                b'k' => black_castling[0] = true,
                b'q' => black_castling[1] = true,
                _ => return Err(InvalidCastlingArgumentError(byte)),
            }
        }

        // 4. En Passant Target Square
        let mut step = 0; // 0 = reading files (a-h), 1 = reading ranks (1-8), 2 = finished

        for byte in fen_code_parts[3].bytes() {
            match byte {
                b'-' => {
                    ep_target = -1; // no en passant target square
                    break;
                }
                b'a'..=b'h' => {
                    if step == 0 {
                        ep_target = byte as i8;
                        step += 1;
                    } else {
                        return Err(InvalidEPArgumentError(String::from(fen_code_parts[3])));
                    }
                }
                b'1'..=b'8' => {
                    if step == 1 {
                        ep_target *= byte as i8;
                        step += 1;
                    } else {
                        return Err(InvalidEPArgumentError(String::from(fen_code_parts[3])));
                    }
                }
                _ => return Err(InvalidEPArgumentError(String::from(fen_code_parts[3]))),
            }
        }

        // 5. Halfmove Clock

        if let Ok(num) = fen_code_parts[4].parse::<u8>() {
            if (0..=100).contains(&num) {
                half_clock = num;
            } else {
                return Err(HalfClockArgumentNotInRangeError(num));
            }
        } else {
            return Err(InvalidHalfClockArgumentError(String::from(
                fen_code_parts[4],
            )));
        }

        // 6. Fullove Clock

        if let Ok(num) = fen_code_parts[5].parse::<u16>() {
            if num >= 1 {
                full_clock = num;
            } else {
                return Err(FullClockArgumentNotInRangeError(num));
            }
        } else {
            return Err(InvalidFullClockArgumentError(String::from(
                fen_code_parts[5],
            )));
        }

        // Occupancy Calculations

        Ok(Board {
            piece_bitboards: piece_bitboards,
            white_occu: white_occu,
            black_occu: black_occu,
            full_occu: full_occu,
            white_active: white_active,
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
