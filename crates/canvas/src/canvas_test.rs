use super::*;
    use color::Color;

    #[test]
    fn test_draw_rect() {
        let mut c = Canvas::new();
        c.set_paint(Paint::fill(Color::RED));
        c.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));
        let dl = c.finalize();
        assert_eq!(dl.commands.len(), 1);
    }

    #[test]
    fn test_save_restore_transform() {
        let mut c = Canvas::new();
        c.set_paint(Paint::fill(Color::RED));
        c.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));

        c.save();
        c.translate(50.0, 0.0);
        c.set_paint(Paint::fill(Color::BLUE));
        c.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));
        c.restore();

        c.draw_rect(Rect::new(Vec2::new(200.0, 0.0), Vec2::new(300.0, 100.0)));
        let dl = c.finalize();
        assert_eq!(dl.commands.len(), 3);
    }
