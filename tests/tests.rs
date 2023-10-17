use imgref2::*;

#[test]
fn iter() {
    let img = vec![1u8, 2].into_imgref(1, 2);
    let mut it = img.pixels();
    assert_eq!(Some(1), it.next());
    assert_eq!(Some(2), it.next());
    assert_eq!(None, it.next());

    let buf = [1u8; (16 + 3) * (8 + 1)];
    for width in 1..16 {
        for height in 1..8 {
            for pad in 0..3 {
                let stride = width + pad;
                let img = buf[..stride * height + stride - width].new_stride(width, height, stride);
                assert_eq!(width * height, img.pixels().map(|a| a as usize).sum(), "{width}x{height}");
                assert_eq!(width * height, img.pixels().count(), "{width}x{height}");
                assert_eq!(height, img.rows().count());

                let mut iter1 = img.pixels();
                let mut left = width * height;
                while let Some(_px) = iter1.next() {
                    left -= 1;
                    assert_eq!(left, iter1.len());
                }
                assert_eq!(0, iter1.len()); iter1.next();
                assert_eq!(0, iter1.len());

                let mut iter2 = img.rows();
                match iter2.next() {
                    Some(_) => {
                        assert_eq!(height - 1, iter2.size_hint().0);
                        assert_eq!(height - 1, iter2.filter(|_| true).count());
                    },
                    None => {
                        assert_eq!(height, 0);
                    },
                };
            }
        }
    }
}
