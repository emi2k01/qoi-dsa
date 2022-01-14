use crate::{
    decoder::ChunksDecoder,
    error::DecodeImageError,
    header::{Channels, Header},
    pixel::Pixel,
};

pub struct Image {
    header: Header,
    pixels: Vec<Pixel>,
}

impl Image {
    pub fn from_slice(input: &[u8]) -> Result<Self, DecodeImageError> {
        let header_input = input[0..14]
            .try_into()
            .map_err(|_| DecodeImageError::InvalidLength(input.len()))?;
        let header = Header::decode(header_input)?;

        let size = header.width * header.height;

        let chunks_input = input[14..].iter().copied();
        let pixels = ChunksDecoder::from_iter(chunks_input).decode(size)?;

        Ok(Self { header, pixels })
    }

    fn to_raw_bitmap(&self) -> Vec<u8> {
        let pixels = match self.header.channels {
            Channels::Rgb => self
                .pixels
                .iter()
                .copied()
                .flat_map(|p| [p.r, p.g, p.b])
                .collect(),
            Channels::Rgba => self
                .pixels
                .iter()
                .copied()
                .flat_map(|p| [p.r, p.g, p.b, p.a])
                .collect(),
        };

        pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! decode_image {
        ($img_name:expr) => {{
            let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $img_name, ".qoi");
            let img = std::fs::read(&img_path).unwrap();
            Image::from_slice(&img).unwrap()
        }};
    }

    macro_rules! save_snapshot {
        ($img_name:expr, $pixels:expr) => {{
            std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/snapshots/")).unwrap();
            std::fs::write(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/snapshots/",
                    $img_name,
                    ".data"
                ),
                $pixels,
            )
            .unwrap();
        }};
    }

    macro_rules! read_snapshot {
        ($img_name:expr) => {{
            std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/snapshots/",
                $img_name,
                ".data"
            ))
            .ok()
        }};
    }

    macro_rules! test_snapshot {
        ($img_name:expr) => {{
            let img = decode_image!($img_name);
            let bitmap = img.to_raw_bitmap();

            if let Some(snapshot) = read_snapshot!($img_name) {
                if snapshot != bitmap {
                    panic!("decoded value does not match snapshot");
                }
            } else {
                save_snapshot!($img_name, bitmap);
            }
        }};
    }

    #[test]
    fn test_image_decode_dice() {
        test_snapshot!("dice");
    }

    #[test]
    fn test_image_decode_kodim10() {
        test_snapshot!("kodim10");
    }

    #[test]
    fn test_image_decode_kodim23() {
        test_snapshot!("kodim23");
    }

    #[test]
    fn test_image_decode_qoi_logo() {
        test_snapshot!("qoi_logo");
    }

    #[test]
    fn test_image_decode_testcard() {
        test_snapshot!("testcard");
    }

    #[test]
    fn test_image_decode_testcard_rgba() {
        test_snapshot!("testcard_rgba");
    }

    #[test]
    fn test_image_decode_wikipedia_008() {
        test_snapshot!("wikipedia_008");
    }
}
