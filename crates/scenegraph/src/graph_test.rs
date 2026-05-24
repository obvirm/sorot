use super::*;
    use color::Color;

    #[test]
    fn test_store_path() {
        let mut g = SceneGraph::new();
        let p = Path::circle(Vec2::new(50.0, 50.0), 30.0);
        let id = g.store_path(&p);
        let stored = g.get_path(id).unwrap();
        assert_eq!(stored.verbs.len(), p.verb_count());
    }

    #[test]
    fn test_add_node() {
        let mut g = SceneGraph::new();
        let paint = g.add_paint(Paint::fill(Color::RED));
        let rect = g.add_node(
            NodeKind::Rect { rect: Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)) },
            paint,
        );
        assert_eq!(rect, 0);
        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn test_hierarchy() {
        let mut g = SceneGraph::new();
        let p = g.add_paint(Paint::fill(Color::BLACK));
        let parent = g.add_node(NodeKind::Group { opacity: 1.0 }, p);
        let child = g.add_node(
            NodeKind::Rect { rect: Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)) },
            p,
        );
        g.add_child(parent, child);
        assert_eq!(g.nodes[parent as usize].first_child, child);
        assert_eq!(g.nodes[child as usize].parent, parent);
    }
