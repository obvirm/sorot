use super::*;

    #[test]
    fn test_from_rgba_u8() {
        let c = Color::from_rgba_u8(255, 0, 0, 128);
        let rgba = c.to_rgba_u8();
        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 0);
        assert_eq!(rgba[2], 0);
        assert_eq!(rgba[3], 128);
    }

    #[test]
    fn test_over_transparent() {
        let src = Color::from_rgba(1.0, 0.0, 0.0, 0.5);
        let dst = Color::TRANSPARENT;
        let result = src.over(dst);
        assert!(result.r > 0.0);
        assert!(result.a > 0.0);
    }

    #[test]
    fn test_over_opaque_bg() {
        let src = Color::from_rgba(1.0, 0.0, 0.0, 0.5);
        let dst = Color::WHITE;
        let result = src.over(dst);
        assert!(result.r > 0.5);
        assert!((result.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_premultiplied_identity() {
        let c = Color::from_rgba(1.0, 1.0, 1.0, 0.0);
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 0.0);
    }

    #[test]
    fn test_unpremultiply_roundtrip() {
        let original = Color::from_rgba(0.5, 0.25, 0.75, 0.8);
        let (r, g, b, a) = original.unpremultiply();
        let rebuilt = Color::from_rgba(r, g, b, a);
        assert!((rebuilt.r - original.r).abs() < 1e-6);
        assert!((rebuilt.g - original.g).abs() < 1e-6);
        assert!((rebuilt.b - original.b).abs() < 1e-6);
    }
