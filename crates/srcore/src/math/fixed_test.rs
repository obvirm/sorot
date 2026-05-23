use super::*;
    #[test] fn test_mul() { assert!((fixed_to_float(fixed_mul(float_to_fixed(2.5), float_to_fixed(3.0))) - 7.5).abs() < 0.01); }
    #[test] fn test_div() { assert!((fixed_to_float(fixed_div(float_to_fixed(7.5), float_to_fixed(2.5))) - 3.0).abs() < 0.01); }
