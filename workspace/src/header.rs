pub const MAGIC: [u8; 4] = [b'q', b'o', b'i', b'f'];

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub width: u32,
    pub height: u32,
    pub channels: Channels,
    colorspace: Colorspace,
}

impl Header {
    pub fn decode(input: [u8; 14]) -> Result<Self, DecodeHeaderError> {
        if input[0..4] != MAGIC {
            return Err(DecodeHeaderError::Magic([
                input[0], input[1], input[2], input[3],
            ]));
        }

        let width = u32::from_be_bytes([input[4], input[5], input[6], input[7]]);
        let height = u32::from_be_bytes([input[8], input[9], input[10], input[11]]);

        let channels = input[12];
        let channels = match channels {
            3 => Channels::Rgb,
            4 => Channels::Rgba,
            _ => return Err(DecodeHeaderError::Channels(channels)),
        };

        let colorspace = input[13];
        let colorspace = match colorspace {
            0 => Colorspace::Srgb,
            1 => Colorspace::Linear,
            _ => return Err(DecodeHeaderError::Colorspace(colorspace)),
        };

        Ok(Self {
            width,
            height,
            channels,
            colorspace,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Rgb = 3,
    Rgba = 4,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Colorspace {
    Srgb,
    Linear,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! decode_header {
        ($img:expr) => {{
            let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $img);
            let img = std::fs::read(&img_path).unwrap();
            let header_section = img[0..14].try_into().unwrap();
            Header::decode(header_section).unwrap()
        }};
    }

    #[test]
    fn test_header_decode_dice() {
        let result = decode_header!("dice.qoi");

        let expected = Header {
            width: 800,
            height: 600,
            channels: Channels::Rgba,
            colorspace: Colorspace::Srgb,
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn test_header_decode_testcard() {
        let result = decode_header!("testcard.qoi");

        let expected = Header {
            width: 256,
            height: 256,
            channels: Channels::Rgba,
            colorspace: Colorspace::Srgb,
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn test_header_decode_testcard_rgba() {
        let result = decode_header!("testcard_rgba.qoi");

        let expected = Header {
            width: 256,
            height: 256,
            channels: Channels::Rgba,
            colorspace: Colorspace::Srgb,
        };

        assert_eq!(result, expected)
    }
}
