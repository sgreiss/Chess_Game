use crate::board::FenParseError;
use crate::board::FenParseError::*;
use crate::board::Piece;
use crate::board::piece;

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

        for i in 0..6 {
            let w_index = 2 * i;        // only even indices for white pieces
            let b_index = 2 * i + 1;    // only odd indices for black pieces
            white_occu |= piece_bitboards[w_index];
            black_occu |= piece_bitboards[b_index];
        }

        full_occu |= white_occu;
        full_occu |= black_occu;

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
