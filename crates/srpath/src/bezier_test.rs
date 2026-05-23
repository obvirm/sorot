use super::*;

    #[test]
    fn test_cubic_eval() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(4.0, 0.0),
        );
        let start = c.eval(0.0);
        let end = c.eval(1.0);
        let mid = c.eval(0.5);
        assert!((start.x - 0.0).abs() < 1e-6);
        assert!((end.x - 4.0).abs() < 1e-6);
        assert!((mid.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_split() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(3.0, 0.0),
        );
        let (left, right) = c.split(0.5);
        let mid = c.eval(0.5);
        assert!((left.p3 - mid).length() < 1e-6);
        assert!((right.p0 - mid).length() < 1e-6);
    }

    #[test]
    fn test_cubic_bbox() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(3.0, -1.0),
            Vec2::new(4.0, 0.0),
        );
        let bbox = c.bounding_box();
        assert!(bbox.min.x <= 0.0);
        assert!(bbox.max.x >= 4.0);
    }

    #[test]
    fn test_quad_to_cubic() {
        let q = Quad::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(2.0, 0.0),
        );
        let c = q.to_cubic();
        let diff_start = (q.eval(0.0) - c.eval(0.0)).length();
        let diff_mid = (q.eval(0.5) - c.eval(0.5)).length();
        let diff_end = (q.eval(1.0) - c.eval(1.0)).length();
        assert!(diff_start < 1e-6);
        assert!(diff_mid < 1e-6);
        assert!(diff_end < 1e-6);
    }
