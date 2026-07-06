use crate::board::FenParseError;
use crate::board::FenParseError::InvalidPieceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
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
