use super::*;
    #[test] fn test_ops() { let a = Vec2::new(1.0, 2.0); let b = Vec2::new(3.0, 4.0); assert_eq!(a + b, Vec2::new(4.0, 6.0)); assert_eq!(a.dot(b), 11.0); assert_eq!(a.cross(b), -2.0); }
    #[test] fn test_perp() { assert_eq!(Vec2::new(1.0, 0.0).perp(), Vec2::new(0.0, 1.0)); }
