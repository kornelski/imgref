# 2D slice of a `Vec`

This is a lowest common denominator struct for working with image fragments in Rust code. It represents a 2-dimensional vector and rectangular slices of it.

* [API Reference](https://docs.rs/imgref)
* [Installation](https://crates.io/crates/imgref)

In graphics code it's very common to pass `width` and `height` along with a `Vec` of pixels — all as separate arguments. This gets very repetitive, and can lead to errors.

This crate is a simple struct that adds dimensions to the underlying buffer. This makes it easier to correctly keep track of the image size and allows passing images with just one function argument instead three or four.

Additionally, it has a concept of a `stride`, which allows defining sub-regions of images without copying, as well as padding (e.g. buffers for video frames may require to be a multiple of 8, regardless of logical image size).

For convenience, it implements iterators for pixels/rows and indexing with `img[(x,y)]`.

```rust
extern crate imgref;
use imgref::*;

fn main() {
    let img = Img::new(vec![0; 4], 2, 2);

    let new_image = some_image_processing_function(img.as_ref());

    println("New size is {}×{}", new_image.width(), new_image.height());
    println("And the top left pixel is {:?}", new_image[(0,0)]);

    for row in new_image.rows() {
        …
    }
    for px in new_image.pixels() {
        …
    }
}
```

## Type aliases

This is described in [more detail in the reference](https://docs.rs/imgref).

### `ImgVec`

It owns its pixels (held in a `Vec`). It's analogous to a 2-dimensional `Vec`. Use this type to create and return new images from functions.

Don't use `&ImgVec`. Instead call `ImgVec.as_ref()` to get a reference (`ImgRef`) from it (explicit `.as_ref()` call is required, because Rust doesn't support [custom conversions](https://github.com/rust-lang/rfcs/pull/1524) yet.)

### `ImgRef`

`ImgRef` is a reference to pixels owned by some other `ImgVec` or a slice. It's analogous to a 2-dimensional `&[]`.

Use this type to accept read-only images as arguments in functions. Note that `ImgRef` is a `Copy` type. Pass `ImgRef`, and *not* `&ImgRef`.
