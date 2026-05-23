use super::*;

    #[test]
    fn test_identity() {
        let m = Matrix::identity();
        assert!(m.is_identity());
        assert_eq!(m.map_point(Vec2::new(5.0, 3.0)), Vec2::new(5.0, 3.0));
    }

    #[test]
    fn test_translate() {
        let m = Matrix::translate(10.0, 20.0);
        assert!(m.is_translation_only());
        assert_eq!(m.map_point(Vec2::new(1.0, 2.0)), Vec2::new(11.0, 22.0));
    }

    #[test]
    fn test_scale() {
        let m = Matrix::scale(2.0, 3.0);
        assert_eq!(m.map_point(Vec2::new(1.0, 1.0)), Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_rotate() {
        let m = Matrix::rotate(std::f32::consts::PI);
        let p = m.map_point(Vec2::new(1.0, 0.0));
        assert!((p.x + 1.0).abs() < 1e-5);
        assert!(p.y.abs() < 1e-5);
    }

    #[test]
    fn test_then() {
        let m = Matrix::translate(5.0, 0.0).then(Matrix::scale(2.0, 1.0));
        assert_eq!(m.map_point(Vec2::new(1.0, 1.0)), Vec2::new(7.0, 1.0));
    }

    #[test]
    fn test_inverse() {
        let m = Matrix::translate(10.0, 20.0).then(Matrix::scale(2.0, 3.0));
        let inv = m.inverse().unwrap();
        let p = Vec2::new(5.0, 6.0);
        let rt = inv.map_point(m.map_point(p));
        assert!((rt.x - p.x).abs() < 1e-4);
    }

    #[test]
    fn test_map_vector() {
        let m = Matrix::translate(10.0, 20.0).then(Matrix::scale(2.0, 1.0));
        assert_eq!(m.map_vector(Vec2::new(3.0, 4.0)), Vec2::new(6.0, 4.0));
    }

    #[test]
    fn test_decompose() {
        let (sx, sy) = Matrix::scale(2.0, 3.0).decompose_scale();
        assert!((sx - 2.0).abs() < 1e-5);
        assert!((sy - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_map_points() {
        let m = Matrix::translate(1.0, 2.0);
        let mut pts = vec![Vec2::new(3.0, 4.0), Vec2::new(5.0, 6.0)];
        m.map_points(&mut pts);
        assert_eq!(pts[0], Vec2::new(4.0, 6.0));
    }
