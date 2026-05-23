use super::*;

    #[test]
    fn test_distance_to_segment_endpoint() {
        let d = distance_to_segment(
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
        );
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance_to_segment_midpoint() {
        let d = distance_to_segment(
            Vec2::new(5.0, 3.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        );
        assert!((d - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_winding_square() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        assert_eq!(winding_number(Vec2::new(5.0, 5.0), &points), 1);
        assert_eq!(winding_number(Vec2::new(20.0, 20.0), &points), 0);
    }

    #[test]
    fn test_sdf_alpha_zero() {
        assert_eq!(sdf_alpha(-1.0, 0.5), 0.0);
    }

    #[test]
    fn test_sdf_alpha_one() {
        assert_eq!(sdf_alpha(1.0, 0.5), 1.0);
    }
