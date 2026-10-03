//! Golden-image parity through the public CPU compositor.

use super::fixtures::{data, parity_appearance, read_sources, root};
use kestrum::portrait_rendering::PortraitCompositor;
use macroquad_toolkit::assets::decode_image_bytes;
use std::fs;

#[test]
fn frozen_csharp_reference_matches_rust_composites_at_128_and_256() {
    let data = data();
    let mut compositor = PortraitCompositor::new(&data.portraits).expect("build art manifest");
    assert!(compositor.enabled());
    let compressed = read_sources(&compositor);
    let appearance = parity_appearance();

    for (size, reference) in [
        (
            128,
            "assets/art-source/portraits/illustrated/parity_v1_128.png",
        ),
        (
            256,
            "assets/art-source/portraits/illustrated/parity_v1_256.png",
        ),
    ] {
        let actual = compositor
            .compose(&compressed, &appearance, size)
            .unwrap_or_else(|error| panic!("compose portrait at {size}px: {error}"));
        let reference_path = root().join(reference);
        let reference_bytes = fs::read(&reference_path).unwrap_or_else(|error| {
            panic!("read C# reference {}: {error}", reference_path.display())
        });
        let expected = decode_image_bytes(&reference_bytes, None)
            .unwrap_or_else(|error| panic!("decode C# reference at {size}px: {error}"));

        assert_eq!((actual.width, actual.height), (size, size));
        assert_eq!((expected.width, expected.height), (size, size));
        assert_eq!(actual.bytes.len(), expected.bytes.len());
        let mut max_alpha_delta = 0_u8;
        let mut max_premultiplied_delta = 0.0_f64;
        let (actual_pixels, actual_tail) = actual.bytes.as_chunks::<4>();
        let (expected_pixels, expected_tail) = expected.bytes.as_chunks::<4>();
        assert!(actual_tail.is_empty() && expected_tail.is_empty());

        for (actual, expected) in actual_pixels.iter().zip(expected_pixels) {
            max_alpha_delta = max_alpha_delta.max(actual[3].abs_diff(expected[3]));
            for channel in 0..3 {
                let actual_value = f64::from(actual[channel]) * f64::from(actual[3]) / 255.0;
                let expected_value = f64::from(expected[channel]) * f64::from(expected[3]) / 255.0;
                max_premultiplied_delta =
                    max_premultiplied_delta.max((actual_value - expected_value).abs());
            }
        }

        assert!(
            max_alpha_delta <= 1,
            "{size}px alpha delta {max_alpha_delta} exceeds one byte"
        );
        assert!(
            max_premultiplied_delta <= 1.0,
            "{size}px premultiplied channel delta {max_premultiplied_delta:.6} exceeds one byte unit"
        );
    }
}
