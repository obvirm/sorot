use super::*;
    #[test] fn test_intersects() { let r1 = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)); assert!(r1.intersects(Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0)))); }
    #[test] fn test_intersect() { let i = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)).intersect(Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0))); assert_eq!(i.min, Vec2::new(5.0, 5.0)); }
