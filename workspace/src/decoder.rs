use crate::{error::DecodeChunksError, pixel::Pixel};

const END_MARKER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

pub struct ChunksDecoder<I> {
    input: I,
    seen_pixels: [Pixel; 64],
    last_pixel: Pixel,
    pixels: Vec<Pixel>,
}

impl<I> ChunksDecoder<I>
where
    I: Iterator<Item = u8>,
{
    pub fn from_iter(input: I) -> Self {
        Self {
            input,
            seen_pixels: [Pixel::zero(); 64],
            last_pixel: Pixel::black(),
            pixels: Vec::new(),
        }
    }

    pub fn decode(mut self, size: u32) -> Result<Vec<Pixel>, DecodeChunksError> {
        while (self.pixels.len() as u32) < size {
            if let Some(tag) = self.input.next() {
                match tag {
                    0b11111110 => {
                        self.decode_op_rgb()?;
                    }

                    0b11111111 => {
                        self.decode_op_rgba()?;
                    }

                    0b00000000..=0b00111111 => {
                        self.decode_op_index(tag);
                    }

                    0b01000000..=0b01111111 => {
                        self.decode_op_diff(tag);
                    }

                    0b10000000..=0b10111111 => {
                        self.decode_op_luma(tag)?;
                    }

                    0b11000000.. => {
                        self.decode_op_run(tag);
                    }
                }

                self.last_pixel = self.pixels.last().copied().unwrap();
                self.see_pixel();
            } else {
                return Err(DecodeChunksError::UnexpectedInputEnd);
            }
        }

        if !self.input.eq(END_MARKER) {
            return Err(DecodeChunksError::BadEnding);
        }

        Ok(self.pixels)
    }

    fn decode_op_rgb(&mut self) -> Result<(), DecodeChunksError> {
        let r = self.take_byte()?;
        let g = self.take_byte()?;
        let b = self.take_byte()?;
        let a = self.last_pixel.a;

        self.pixels.push(Pixel { r, g, b, a });

        Ok(())
    }

    fn decode_op_rgba(&mut self) -> Result<(), DecodeChunksError> {
        let r = self.take_byte()?;
        let g = self.take_byte()?;
        let b = self.take_byte()?;
        let a = self.take_byte()?;

        self.pixels.push(Pixel { r, g, b, a });

        Ok(())
    }

    fn decode_op_index(&mut self, tag: u8) {
        let index = tag as usize;
        let seen_pixel = self.seen_pixels[index];

        self.pixels.push(seen_pixel);
    }

    fn decode_op_diff(&mut self, tag: u8) {
        let mut pixel = self.last_pixel;

        let delta_r = (tag & 0b00110000) >> 4;
        let delta_g = (tag & 0b00001100) >> 2;
        let delta_b = tag & 0b00000011;

        pixel.r = pixel.r.wrapping_add(delta_r).wrapping_sub(2);
        pixel.g = pixel.g.wrapping_add(delta_g).wrapping_sub(2);
        pixel.b = pixel.b.wrapping_add(delta_b).wrapping_sub(2);

        self.pixels.push(pixel);
    }

    fn decode_op_luma(&mut self, tag: u8) -> Result<(), DecodeChunksError> {
        let byte1 = self.take_byte()?;
        let mut pixel = self.last_pixel;

        let delta_g = (tag & 0b00111111).wrapping_sub(32);
        let delta_r = ((byte1 & 0b11110000) >> 4)
            .wrapping_add(delta_g)
            .wrapping_sub(8);
        let delta_b = (byte1 & 0b00001111).wrapping_add(delta_g).wrapping_sub(8);

        pixel.r = pixel.r.wrapping_add(delta_r);
        pixel.g = pixel.g.wrapping_add(delta_g);
        pixel.b = pixel.b.wrapping_add(delta_b);

        self.pixels.push(pixel);

        Ok(())
    }

    fn decode_op_run(&mut self, tag: u8) {
        let run_length = (tag & 0b00111111).wrapping_add(1) as usize;
        let pixel = self.last_pixel;

        for _ in 0..run_length {
            self.pixels.push(pixel);
        }
    }

    fn see_pixel(&mut self) {
        let pixel = self.last_pixel;

        let r = pixel.r as usize;
        let g = pixel.g as usize;
        let b = pixel.b as usize;
        let a = pixel.a as usize;

        let index = (r * 3 + g * 5 + b * 7 + a * 11) % 64;

        self.seen_pixels[index] = pixel;
    }

    fn last_pixel(&self) -> Pixel {
        self.pixels.last().copied().unwrap_or_else(Pixel::black)
    }

    fn take_byte(&mut self) -> Result<u8, DecodeChunksError> {
        self.input
            .next()
            .ok_or(DecodeChunksError::UnexpectedInputEnd)
    }
}
