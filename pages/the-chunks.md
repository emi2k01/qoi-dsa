# The chunks

The specification mentions

> A QOI file consists of a 14-byte header, followed by any number of
data “chunks” and an 8-byte end marker.

We already wrote the header decoder so we're going to do the data chunks decoder
next. But first we need a bit more context on the data chunks format.

Reading the [spec.] we find some details that need to know before we start
writing the decoder. I'll save you a bit of time and lay them here:

1. Images are encoded row by row, left to right, top to bottom.
1. The previous pixel starts with a value of `{r: 0, g: 0, b: 0, a: 255}`.
1. A zero-initialized array with 64 slots of previously seen pixels is maintained.
1. Each chunk starts with a 2 or 8-bit tag followed by its data bits.

Now that we know that we can start writing our decoder.

## Creating our `ChunksDecoder`

We create the [`src/decoder.rs`] file that will contain our chunks decoder.

[`struct ChunksDecoder`] is the structure that will maintain the state of
the decoder. It contains the following fields:

- `input`: Our stream of data chunks.
- `seen_pixels`: An array of previously seen pixels as mentioned by the spec.
- `last_pixel`: The last pixel we decoded.
- `pixels`: The pixels we have decoded.

We then write our [`impl ChunksDecoder`]. Notice that we add a
requirement on the type parameter `I`: It needs to implement the trait
`Iterator` and its `Item` type parameter must be a `u8`. That is, our type `I`
must be an iterator of bytes. This means all the methods in this `impl` block
will only be available if `I: Iterator<Item = u8>` is satisfied.

The constructor will be [`ChunksDecoder::from_iter`]. We initialize two fields
as required by the spec:

- `seen_pixels` is initialized with 64 copies of [`struct Pixel`] created using
  [`Pixel::zero`].
- `last_pixel` is initialized with a [`struct Pixel`] created using
  [`Pixel::black`].

## Entry point

The entry point to our decoding logic is the [`ChunksDecoder::decode`]
function.

It takes a `size` parameter indicating the size in pixels of the image to be
decoded as indicated by the header.

We then start our [main loop]. It will repeat until we have decoded all the
pixels (or we get an error).

## Main loop

Now in the main loop, we start decoding the chunks. But first, lets consult the
spec for a bit more information:

> Each chunk starts with a 2- or 8-bit tag, followed by a number of data bits.
> [...] The 8-bit tags have
precedence over the 2-bit tags.

So, we try to [get the next byte]. If we get the byte, we start matching it
against the tags stated by the spec. If we don't get the byte, [we return an error][no next byte].

## Recognizing the chunks

After we [get the next byte], is when we start decoding the chunks but before
that, lets consult the spec.

The spec mentions 6 types of data chunks, we will go one by one.

### QOI\_OP\_RGB

The first type of chunk is identified as `QOI_OP_RGB`:

```
.- QOI_OP_RGB ------------------------------------------.
|         Byte[0]         | Byte[1] | Byte[2] | Byte[3] |
|  7  6  5  4  3  2  1  0 | 7 .. 0  | 7 .. 0  | 7 .. 0  |
|-------------------------+---------+---------+---------|
|  1  1  1  1  1  1  1  0 |   red   |  green  |  blue   |
`-------------------------------------------------------`
8-bit tag b11111110
8-bit   red channel value
8-bit green channel value
8-bit  blue channel value

The alpha value remains unchanged from the previous pixel.
```

The tag is `11111110`<sub>2</sub> so we [match our tag against it][match
qoi_op_rgb] and if it matches we call [`ChunksDecoder::decode_op_rgb`]

After that, we use [`ChunksDecoder::take_byte`] to get the next 3 bytes which
correspond to the red, green and blue components respectively. We get the alpha
component from the previous pixel.

Then we push a new pixel to our `pixels` field.

### QOI\_OP\_RGBA

```
.- QOI_OP_RGBA ---------------------------------------------------.
|         Byte[0]         | Byte[1] | Byte[2] | Byte[3] | Byte[4] |
|  7  6  5  4  3  2  1  0 | 7 .. 0  | 7 .. 0  | 7 .. 0  | 7 .. 0  |
|-------------------------+---------+---------+---------+---------|
|  1  1  1  1  1  1  1  1 |   red   |  green  |  blue   |  alpha  |
`-----------------------------------------------------------------`
8-bit tag b11111111
8-bit   red channel value
8-bit green channel value
8-bit  blue channel value
8-bit alpha channel value
```

This type of chunk is like the previous one, except that the tag changes and the alpha component is
now part of the data bits.

We [match our tag against 0b11111111][match qoi_op_rgba] and if it matches we call [`ChunksDecoder::decode_op_rgba`].

As I said, this is exactly like the previous type of chunk but now instead of
getting the alpha component from the previous pixel, we get it from the data bits.

### QOI\_OP\_INDEX

```
.- QOI_OP_INDEX ----------.
|         Byte[0]         |
|  7  6  5  4  3  2  1  0 |
|-------+-----------------|
|  0  0 |     index       |
`-------------------------`
2-bit tag b00
6-bit index into the color index array: 0..63
```

This type of chunk indicates that the next pixel is a copy of a previously seen
pixel stored in our [`seen_pixels`] array. The index is stored in the lower 6
six bits of the tag.

In this case, the tag is not a single possible value as it was the case for the
past type of chunks. The first 2 bits are always zeroes, but the lower 6 bits
range from `0b000000` to `0b111111`. The full range would be
`0b00000000..=0b00111111`.

So we need to [match the tag against the range of possible values][match
qoi_op_index]. If it matches, we call [`ChunksDecoder::decode_op_index`] and we
pass `tag` because we still need to extract the index out of it.

Because the 2 most significant bits of `tag` are already zero, we can just use
`tag` as the index. We'll see in the next types of chunks that this won't
always be so easy.

Before indexing `self.seen_pixels` we first have to [cast `tag` to a usize].
Then we can access `self.seen_pixels` to get the pixel. The pixel will be
copied because we derived `Copy` for [`struct Pixel`].

After we get the pixel from `self.seen_pixels`, we add it to `self.pixels`.

### QOI\_OP\_DIFF

```
.- QOI_OP_DIFF -----------.
|         Byte[0]         |
|  7  6  5  4  3  2  1  0 |
|-------+-----+-----+-----|
|  0  1 |  dr |  dg |  db |
`-------------------------`
2-bit tag b01
2-bit   red channel difference from the previous pixel between -2..1
2-bit green channel difference from the previous pixel between -2..1
2-bit  blue channel difference from the previous pixel between -2..1
The difference to the current channel values are using a wraparound operation,
so "1 - 2" will result in 255, while "255 + 1" will result in 0.
Values are stored as unsigned integers with a bias of 2. E.g. -2 is stored as
0 (b00). 1 is stored as 3 (b11).
The alpha value remains unchanged from the previous pixel.
```

This type of chunks indicates that the next pixel is stored as a difference of
the previous pixel.

Same as before, [if it matches][match qoi_op_diff] we call [`ChunksDecoder::decode_op_diff`].

Then we need to extract the differences of the components (`dr`, `dg`, `db`).
For this, we use ["bitmasking"]

Lets see the [extraction of `dr`], for example:

As shown by the figure, `dr`'s bits are stored in the positions 5 and 4 so our
mask will be: `0b00110000`. We put `1` where the bits we want are.

Then we do `tag & 0b00110000`, this will zero out the bits that are also zero
in our mask and leave the rest as they are.

After that, we do `>> 4` to shift the bits for positions to the right so that
the bits end up at the right end. Then we can interpret the result as a normal number.

We repeat this process for `dg` and `db`, changing the mask and the shift
operand.

The spec says the differences are applied using wraparound operations, so we
use `wrapping_add`. It also says the values are stored with a bias of 2, so we
use `wrapping_sub` to apply it.

We apply the differences to a copy of the last seen pixel and push it to
`self.pixels`.

## QOI\_OP\_LUMA

```
.- QOI_OP_LUMA -------------------------------------.
|         Byte[0]         |         Byte[1]         |
|  7  6  5  4  3  2  1  0 |  7  6  5  4  3  2  1  0 |
|-------+-----------------+-------------+-----------|
|  1  0 |  green diff     |   dr - dg   |  db - dg  |
`---------------------------------------------------`
2-bit tag b10
6-bit green channel difference from the previous pixel -32..31
4-bit   red channel difference minus green channel difference -8..7
4-bit  blue channel difference minus green channel difference -8..7
The green channel is used to indicate the general direction of change and is
encoded in 6 bits. The red and blue channels (dr and db) base their diffs off
of the green channel difference and are encoded in 4 bits. I.e.:
	dr_dg = (cur_px.r - prev_px.r) - (cur_px.g - prev_px.g)
	db_dg = (cur_px.b - prev_px.b) - (cur_px.g - prev_px.g)
The difference to the current channel values are using a wraparound operation,
so "10 - 13" will result in 253, while "250 + 7" will result in 1.
Values are stored as unsigned integers with a bias of 32 for the green channel
and a bias of 8 for the red and blue channel.
The alpha value remains unchanged from the previous pixel.
```

We follow the same process as the previous chunk.

## QOI\_OP\_RUN

```
.- QOI_OP_RUN ------------.
|         Byte[0]         |
|  7  6  5  4  3  2  1  0 |
|-------+-----------------|
|  1  1 |       run       |
`-------------------------`
2-bit tag b11
6-bit run-length repeating the previous pixel: 1..62
The run-length is stored with a bias of -1. Note that the run-lengths 63 and 64
(b111110 and b111111) are illegal as they are occupied by the QOI_OP_RGB and
QOI_OP_RGBA tags.
```

We do the same as the previous chunk.

# After decoding a chunk

We store the [new pixel in `self.last_pixel`][new last_pixel] and also [store
it `self.seen_pixels`][new seen_pixel]. But how do we figure out the index of
the new seen pixel?

The spec mentions we use the following function:

```
index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
```

This is what [`ChunksDecoder::see_pixel`] does.

# After reading all pixels

After we read all `size` pixels, we need to check if the rest of the input is
an end marker. The spec states that an end marker is seven 0s followed by a
single 1. We store the end marker in [`const END_MARKER`].

If the [rest of the input is not and end marker][end marker eq] we return a
[`DecodeChunksError::BadEnding`] error. Otherwise, we return the pixels we
decoded.

# Putting it all together

In [`src/image.rs`] we put it all together.

# Testing

We compare the raw pixel data to a snapshot of it. If the snapshot doesn't
exist, we create it and we get an error if it later changes.

To make sure the raw data was correct, I stored the snapshots with the
extension `data` and used gimp to see with my own eyes.

The snapshots are stored in `snapshots/`.

Here's an image of gimp opening a snapshot:

![gimp opening a raw image]({{ static_resource("gimp-opening-a-raw-image.png") }})

[spec.]: <https://qoiformat.org/qoi-specification.pdf>
["bitmasking"]: <https://en.wikipedia.org/wiki/Mask_(computing)>
[main loop]: <#csai:highlight file="src/decoder.rs" from="\s{8}while.* \< size" to="\\{$">
[get the next byte]: <#csai:highlight file="src/decoder.rs" from="\s{8}if let Some\\(tag\\) = self\.input.next" to="\\{$">
[no next byte]: <#csai:highlight file="src/decoder.rs" from="\s{12}return Err\\(.*UnexpectedInputEnd" to=";$">
[match qoi_op_rgb]: <#csai:highlight file="src/decoder.rs" from="\s{20}0b11111110 =\> \\{" to="\s{20}\\}$">
[match qoi_op_rgba]: <#csai:highlight file="src/decoder.rs" from="\s{20}0b11111111 =\> \\{" to="\s{20}\\}$">
[match qoi_op_index]: <#csai:highlight file="src/decoder.rs" from="\s{20}0b00000000\.\.=" to="\s{20}\\}$">
[match qoi_op_diff]: <#csai:highlight file="src/decoder.rs" from="\s{20}0b01000000\.\.=" to="\s{20}\\}$">
[cast `tag` to a usize]: <#csai:highlight file="src/decoder.rs" from="\s{8}let index = tag as usize" to=";">
[extraction of `dr`]: <#csai:highlight file="src/decoder.rs" from="\s{8}let delta_r = \\(tag &" to=";">

[`src/decoder.rs`]: <#csai:open_file file="src/decoder.rs">

[`struct ChunksDecoder`]: <#csai:highlight file="src/decoder.rs" from="pub struct ChunksDecoder" to="^\\}">
[`impl ChunksDecoder`]: <#csai:highlight file="src/decoder.rs" from="impl\<I. ChunksDecoder" to="^\\{">
[`ChunksDecoder::from_iter`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn from_iter" to="^\s{4}\\}">
[`struct Pixel`]: <#csai:highlight file="src/pixel.rs" from="pub struct Pixel" to="^\\}">
[`Pixel::zero`]: <#csai:highlight file="src/pixel.rs" from="\s{4}.*fn zero" to="^\\}">
[`Pixel::black`]: <#csai:highlight file="src/pixel.rs" from="\s{4}.*fn black" to="^\\}">
[`ChunksDecoder::decode`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn decode" to="\\{$">
[`ChunksDecoder::decode_op_rgb`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn decode_op_rgb\\(" to="\\{$">
[`ChunksDecoder::decode_op_rgba`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn decode_op_rgba\\(" to="\\{$">
[`ChunksDecoder::decode_op_index`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn decode_op_index\\(" to="\\{$">
[`ChunksDecoder::decode_op_diff`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn decode_op_diff\\(" to="\\{$">
[`ChunksDecoder::take_byte`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn take_byte" to="\\{$">
[`ChunksDecoder::see_pixel`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*fn see_pixel" to="\\{$">
[`const END_MARKER`]: <#csai:highlight file="src/decoder.rs" from="^const END_MARKER" to=";$">
[`seen_pixels`]: <#csai:highlight file="src/decoder.rs" from="\s{4}.*seen_pixels: \\[" to=",$">
[new last_pixel]: <#csai:highlight file="src/decoder.rs" from="\s{16}self\.last_pixel = " to=";$">
[new seen_pixel]: <#csai:highlight file="src/decoder.rs" from="\s{16}self\.see_pixel\\(\\)" to=";$">
[end marker eq]: <#csai:highlight file="src/decoder.rs" from="^\s{8}if !self.input.eq\\(" to="^\s{8}\\}$">
[`DecodeChunksError::BadEnding`]: <#csai:highlight file="src/error.rs" from="\s{4}BadEnding" to=",$">
[`src/image.rs`]: <#csai:open_file file="src/image.rs">
