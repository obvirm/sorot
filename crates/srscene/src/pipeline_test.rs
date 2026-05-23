use super::*;
    use crate::display_list::DrawRect;
    use srcore::color::Color;

    #[test]
    fn test_build_frame() {
        let mut pipeline = Pipeline::new();
        let graph = SceneGraph::new();
        let mut dl = DisplayList::new();
        dl.commands.push(DrawCommand::Rect(DrawRect {
            rect: Rect::new(Vec2::new(10.0, 10.0), Vec2::new(200.0, 200.0)),
            paint: Paint::fill(Color::BLUE),
            transform: Matrix::identity(),
        }));
        let frame = pipeline.build_frame(&graph, &dl, 800, 600);
        assert!(frame.non_empty_tiles() > 0);
    }
