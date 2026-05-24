use super::*;

    #[test]
    fn test_triangulate_triangle() {
        let mut path = FlattenedPath::new();
        path.verbs.push(FlattenVerb::MoveTo);
        path.points.push(Vec2::new(0.0, 0.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(100.0, 0.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(50.0, 100.0));
        path.verbs.push(FlattenVerb::Close);

        let mesh = triangulate(&path);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_triangulate_square() {
        let mut path = FlattenedPath::new();
        path.verbs.push(FlattenVerb::MoveTo);
        path.points.push(Vec2::new(0.0, 0.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(100.0, 0.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(100.0, 100.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(0.0, 100.0));
        path.verbs.push(FlattenVerb::Close);

        let mesh = triangulate(&path);
        assert_eq!(mesh.indices.len(), 6);
    }

    #[test]
    fn test_triangulate_concave() {
        let mut path = FlattenedPath::new();
        path.verbs.push(FlattenVerb::MoveTo);
        path.points.push(Vec2::new(0.0, 0.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(100.0, 0.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(100.0, 100.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(50.0, 50.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(0.0, 100.0));
        path.verbs.push(FlattenVerb::Close);

        let mesh = triangulate(&path);
        assert!(!mesh.indices.is_empty());
    }
