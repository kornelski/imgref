# 2D slice of a `Vec`

This is the lowest common denominator struct for passiing image fragments around in Rust code.

* [API Reference](https://docs.rs/imgref)
* [Installation](https://crates.io/crates/imgref)

In graphics code it's very common to pass `width` and `height` along with a `Vec` of pixels, all as separate arguments. This is tedious, and can lead to errors.

This crate is a simple struct that adds dimensions to the underlying buffer. This makes it easier to correctly keep track of the image size and allows passing images with just one function argument instead three or four.

Additionally, it has a concept of a `stride`, which allows defining sub-regions of images without copying, as well as padding (e.g. buffers for video frames may require to be a multiple of 8, regardless of logical image size).

For convenience, indexing with `img[(x,y)]` is supported.

```rust
extern crate imgref;
use imgref::*;

fn main() {
    let img = Img::new(vec![0; 4], 2, 2);

    let new_image = resize_image(img.as_ref());

    println("New size is {}x{}", new_image.width(), new_image.height());
    println("And the top left pixel is {:?}", new_image[(0usize,0usize)]);
}
```
