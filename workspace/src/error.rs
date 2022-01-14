#[derive(Debug)]
pub enum DecodeHeaderError {
    Magic([u8; 4]),
    Channels(u8),
    Colorspace(u8),
}

impl std::fmt::Display for DecodeHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Magic(magic) => write!(
                f,
                "invalid magic: `[{:x}, {:x}, {:x}, {:x}]`",
                magic[0], magic[1], magic[2], magic[3]
            ),
            Self::Channels(channels) => write!(f, "invalid channels value: `{}`", channels),
            Self::Colorspace(colorspace) => {
                write!(f, "invalid color space: `{}`", colorspace)
            }
        }
    }
}

impl std::error::Error for DecodeHeaderError {}

#[derive(Debug)]
pub enum DecodeChunksError {
    Io(std::io::Error),
    UnexpectedInputEnd,
    BadEnding,
}

impl std::error::Error for DecodeChunksError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeChunksError::Io(cause) => Some(cause),
            _ => None,
        }
    }
}

impl std::fmt::Display for DecodeChunksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeChunksError::Io(_) => write!(f, "there was an IO error"),
            DecodeChunksError::UnexpectedInputEnd => write!(f, "input ended unexpectedly"),
            DecodeChunksError::BadEnding => write!(f, "input does not mark its end correctly"),
        }
    }
}

impl From<std::io::Error> for DecodeChunksError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug)]
pub enum DecodeImageError {
    InvalidLength(usize),
    SizeMismatch,
    Header(DecodeHeaderError),
    Chunks(DecodeChunksError),
    InvalidDiffOp,
}

impl std::error::Error for DecodeImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeImageError::Header(cause) => Some(cause),
            DecodeImageError::Chunks(cause) => Some(cause),
            _ => None,
        }
    }
}

impl std::fmt::Display for DecodeImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeImageError::InvalidLength(n) => write!(f, "input length is not valid: {}", n),
            DecodeImageError::SizeMismatch => {
                write!(f, "image size does not match header metadata")
            }
            DecodeImageError::Header(_) => write!(f, "header decoding failed"),
            DecodeImageError::Chunks(_) => write!(f, "chunks decoding failed"),
            DecodeImageError::InvalidDiffOp => write!(f, "got diff op as the first op"),
        }
    }
}

impl From<DecodeHeaderError> for DecodeImageError {
    fn from(v: DecodeHeaderError) -> Self {
        Self::Header(v)
    }
}

impl From<DecodeChunksError> for DecodeImageError {
    fn from(e: DecodeChunksError) -> Self {
        Self::Chunks(e)
    }
}
