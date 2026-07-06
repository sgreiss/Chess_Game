use std::fmt;

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
