# The header

Before we start parsing, we need to know the format of what we'll decode. For
that, we'll follow [QOI's specification][qois-spec].

The spec. says:

> A QOI file consists of a 14-byte header, followed by any number of
data “chunks” and an 8-byte end marker.

And the header format is described as:

```c
qoi_header {
    char     magic[4];   // magic bytes "qoif"
    uint32_t width;      // image width in pixels (BE)
    uint32_t height;     // image height in pixels (BE)
    uint8_t  channels;   // 3 = RGB, 4 = RGBA
    uint8_t  colorspace; // 0 = sRGB with linear alpha
                         // 1 = all channels linear
}
```

Notice that, in the comments for the `width` and `height` fields, it says `(BE)`. That means that
the endianness of those fields is big-endian.

## Representing the header

We represent the header with [`struct Header`]. The structure
uses the equivalent data types to those in the C structure shown by the spec.

We store the `magic` bytes in [`const MAGIC`] because there's no need to
store it in every header. We use an array of `u8` because a C `char` is not a
Rust `char`, it is a Rust `u8`; a byte.

## Decoding the header

We write the function [`Header::decode`]. This will be the
constructor for `Header`. In case of error, it returns a
[`struct DecodeHeaderError`].

Then we start decoding the fields:

- **[Magic][magic-decode]**: We compare the first 4 bytes against our [`const MAGIC`]
constant.

  If the first 4 bytes don't match our magic constant, we return
  [`DecodeHeaderError::Magic`] containing the 4 bytes.

- **[Width][width-decode]**: We take bytes with indices 4..=7 to create a `[u8; 4]`. 

  We use `u32::from_be_bytes` because, remember: the `width` and `height`
  fields in the header are in big-endian order.

- **[Height][height-decode]**: For the height we do the same as the width but we
  skip the first first 8 bytes instead.

- **[Channels][channels-decode]**: The `channels` field is a single byte at the
  13th position so we just index it directly. We match its value and return a
  variant of [`enum Channels`] depending on the value.

  If the value is invalid, we return [`DecodeHeaderError::Channels`]
  containing the value we got.

- **[Colorspace][colorspace-decode]**: We get the byte at the 14th position and
  match the value against 0 or 1 to choose the [`enum Colorspace`] variant.

  If the value is invalid, we return [`DecodeHeaderError::Colorspace`]
  containing the value we got.

## Testing

For testing we use the images provided by [qoiformat.org][qoi-format]. These
are the images stored in the `assets/` directory.

We write a [`macro decode_header`] to help us read the images from the
filesystem and pass the first 14 bytes to [`Header::decode`].

All tests have the same structure:

- Decode header from image.
- Create expected result.
- Assert the decoded header matches the expected result.

We run the tests:


```console
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests (target/debug/deps/qoi-21872ddd3645acaa)

running 3 tests
test header::tests::test_header_decode_testcard ... ok
test header::tests::test_header_decode_testcard_rgba ... ok
test header::tests::test_header_decode_dice ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

[qoi-format]: https://qoiformat.org/
[qois-spec]: https://qoiformat.org/qoi-specification.pdf

[`struct Header`]: <#csai:highlight file="src/header.rs" from="^pub struct Header" to="^\\}">
[`const MAGIC`]: <#csai:highlight file="src/header.rs" from="^pub const MAGIC" to="\\];">
[`Header::decode`]: <#csai:highlight file="src/header.rs" from="^\s{4}pub fn decode\\(" to="\\{"> 
[`struct DecodeHeaderError`]: <#csai:highlight file="src/header.rs" from="^pub enum DecodeHeaderError" to="^}"> 
[`DecodeHeaderError::Magic`]: <#csai:highlight file="src/header.rs" from="^\s{4}Magic" to=","> 
[`enum Channels`]: <#csai:highlight file="src/header.rs" from="^pub enum Channels" to="^\}"> 
[`DecodeHeaderError::Channels`]: <#csai:highlight file="src/header.rs" from="^\s{4}Channels" to=",">
[`enum Colorspace`]: <#csai:highlight file="src/header.rs" from="^pub enum Colorspace" to="^\}"> 
[`DecodeHeaderError::Colorspace`]: <#csai:highlight file="src/header.rs" from="^\s{4}Colorspace" to=",">
[`macro decode_header`]: <#csai:highlight file="src/header.rs" from="^\s{4}macro_rules! decode_header" to="^\s{4}\}">

[magic-decode]: <#csai:highlight file="src/header.rs" from="^\s{8}if input.*MAGIC" to="^\s{8}\\}"> 
[width-decode]: <#csai:highlight file="src/header.rs" from="^\s{8}let width" to=";"> 
[height-decode]: <#csai:highlight file="src/header.rs" from="^\s{8}let height" to=";"> 
[channels-decode]: <#csai:highlight file="src/header.rs" from="^\s{8}let channels.*;" to="\s{8}\};"> 
[colorspace-decode]: <#csai:highlight file="src/header.rs" from="^\s{8}let colorspace.*;" to="\s{8}\};"> 

