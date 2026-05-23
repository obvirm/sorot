use super::*;

    #[test]
    fn test_blend_src_over_opaque() {
        let mut dst = [0u8, 0, 0, 0, 0, 0, 0, 0];
        let src = [255u8, 0, 0, 255, 0, 255, 0, 128];
        blend_src_over(&mut dst, &src);
        assert_eq!(dst[0], 255);
        assert_eq!(dst[3], 255);
        assert!(dst[5] > 0);
    }

    #[test]
    fn test_fill_color() {
        let mut buf = vec![0u8; 40];
        fill_color(&mut buf, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 0);
        assert_eq!(buf[3], 255);
        assert_eq!(buf[4], 255);
    }
