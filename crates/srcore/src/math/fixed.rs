pub type Fixed = i32;
pub const FIXED_SHIFT: u32 = 8;
pub const FIXED_ONE: Fixed = 1 << FIXED_SHIFT;
pub const FIXED_HALF: Fixed = FIXED_ONE >> 1;

#[inline] pub fn float_to_fixed(f: f32) -> Fixed { (f * FIXED_ONE as f32) as Fixed }
#[inline] pub fn fixed_to_float(f: Fixed) -> f32 { f as f32 / FIXED_ONE as f32 }
#[inline] pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed { ((a as i64 * b as i64 + FIXED_HALF as i64) >> FIXED_SHIFT) as Fixed }
#[inline] pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed { (((a as i64) << FIXED_SHIFT) / b as i64) as Fixed }
#[inline] pub fn fixed_floor(f: Fixed) -> Fixed { f & !(FIXED_ONE - 1) }
#[inline] pub fn fixed_ceil(f: Fixed) -> Fixed { (f + FIXED_ONE - 1) & !(FIXED_ONE - 1) }
#[inline] pub fn fixed_round(f: Fixed) -> Fixed { (f + FIXED_HALF) & !(FIXED_ONE - 1) }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedMatrix {
    pub a: Fixed, pub c: Fixed, pub e: Fixed,
    pub b: Fixed, pub d: Fixed, pub f: Fixed,
}

impl FixedMatrix {
    #[inline] pub const fn identity() -> Self { Self { a: FIXED_ONE, c: 0, e: 0, b: 0, d: FIXED_ONE, f: 0 } }
    #[inline] pub fn map_point(self, x: Fixed, y: Fixed) -> (Fixed, Fixed) {
        (fixed_mul(self.a, x) + fixed_mul(self.c, y) + self.e, fixed_mul(self.b, x) + fixed_mul(self.d, y) + self.f)
    }
}

impl Default for FixedMatrix { fn default() -> Self { Self::identity() } }

#[cfg(test)]
#[path = "fixed_test.rs"]
mod tests;
