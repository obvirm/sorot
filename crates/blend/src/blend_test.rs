use super::*;

    #[test]
    fn test_src_over() {
        let src = Color::from_rgba(1.0, 0.0, 0.0, 0.5);
        let dst = Color::from_rgba(0.0, 0.0, 1.0, 1.0);
        let result = BlendMode::SrcOver.blend(src, dst);
        assert!(result.r > 0.0);
        assert!(result.b > 0.0);
    }

    #[test]
    fn test_multiply() {
        let src = Color::from_rgba(0.8, 0.8, 0.8, 1.0);
        let dst = Color::from_rgba(0.5, 0.5, 0.5, 1.0);
        let result = BlendMode::Multiply.blend(src, dst);
        assert!(result.r < 0.5);
    }
