use srcore::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathVerb {
    MoveTo,
    LineTo,
    QuadTo,
    CubicTo,
    Close,
}

impl PathVerb {
    #[inline]
    pub fn point_count(self) -> usize {
        match self {
            PathVerb::MoveTo => 1,
            PathVerb::LineTo => 1,
            PathVerb::QuadTo => 2,
            PathVerb::CubicTo => 3,
            PathVerb::Close => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    Winding,
    EvenOdd,
}

#[derive(Debug, Clone)]
pub struct Path {
    verbs: Vec<PathVerb>,
    points: Vec<Vec2>,
}

#[derive(Debug, Clone)]
pub struct PathIter<'a> {
    verbs: &'a [PathVerb],
    points: &'a [Vec2],
    verb_idx: usize,
    point_idx: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct PathSegment<'a> {
    pub verb: PathVerb,
    pub points: &'a [Vec2],
}

impl Path {
    pub fn new() -> Self {
        Self {
            verbs: Vec::new(),
            points: Vec::new(),
        }
    }

    pub fn with_capacity(verbs_cap: usize, points_cap: usize) -> Self {
        Self {
            verbs: Vec::with_capacity(verbs_cap),
            points: Vec::with_capacity(points_cap),
        }
    }

    #[inline]
    pub fn move_to(&mut self, p: Vec2) {
        self.verbs.push(PathVerb::MoveTo);
        self.points.push(p);
    }

    #[inline]
    pub fn line_to(&mut self, p: Vec2) {
        self.verbs.push(PathVerb::LineTo);
        self.points.push(p);
    }

    #[inline]
    pub fn quad_to(&mut self, ctrl: Vec2, to: Vec2) {
        self.verbs.push(PathVerb::QuadTo);
        self.points.push(ctrl);
        self.points.push(to);
    }

    #[inline]
    pub fn cubic_to(&mut self, ctrl1: Vec2, ctrl2: Vec2, to: Vec2) {
        self.verbs.push(PathVerb::CubicTo);
        self.points.push(ctrl1);
        self.points.push(ctrl2);
        self.points.push(to);
    }

    #[inline]
    pub fn close(&mut self) {
        self.verbs.push(PathVerb::Close);
    }

    #[inline]
    pub fn verbs(&self) -> &[PathVerb] {
        &self.verbs
    }

    #[inline]
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    #[inline]
    pub fn verb_count(&self) -> usize {
        self.verbs.len()
    }

    #[inline]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    pub fn iter(&self) -> PathIter<'_> {
        PathIter {
            verbs: &self.verbs,
            points: &self.points,
            verb_idx: 0,
            point_idx: 0,
        }
    }

    /// Rect, Line, RoundedRect, Oval, Arc helpers
    pub fn rect(min: Vec2, max: Vec2) -> Self {
        let mut path = Self::new();
        path.move_to(Vec2::new(min.x, min.y));
        path.line_to(Vec2::new(max.x, min.y));
        path.line_to(Vec2::new(max.x, max.y));
        path.line_to(Vec2::new(min.x, max.y));
        path.close();
        path
    }

    pub fn oval(center: Vec2, rx: f32, ry: f32) -> Self {
        let k = 0.5522847498; // magic constant for circle approximation
        let kx = rx * k;
        let ky = ry * k;

        let mut p = Self::new();
        p.move_to(Vec2::new(center.x + rx, center.y));
        p.cubic_to(
            Vec2::new(center.x + rx, center.y + ky),
            Vec2::new(center.x + kx, center.y + ry),
            Vec2::new(center.x, center.y + ry),
        );
        p.cubic_to(
            Vec2::new(center.x - kx, center.y + ry),
            Vec2::new(center.x - rx, center.y + ky),
            Vec2::new(center.x - rx, center.y),
        );
        p.cubic_to(
            Vec2::new(center.x - rx, center.y - ky),
            Vec2::new(center.x - kx, center.y - ry),
            Vec2::new(center.x, center.y - ry),
        );
        p.cubic_to(
            Vec2::new(center.x + kx, center.y - ry),
            Vec2::new(center.x + rx, center.y - ky),
            Vec2::new(center.x + rx, center.y),
        );
        p.close();
        p
    }

    pub fn circle(center: Vec2, r: f32) -> Self {
        Self::oval(center, r, r)
    }

    pub fn rounded_rect(min: Vec2, max: Vec2, rx: f32, ry: f32) -> Self {
        let rx = rx.min((max.x - min.x) * 0.5);
        let ry = ry.min((max.y - min.y) * 0.5);
        let k = 0.5522847498;
        let kx = rx * k;
        let ky = ry * k;

        let x0 = min.x;
        let y0 = min.y;
        let x1 = max.x;
        let y1 = max.y;

        let mut p = Self::new();
        p.move_to(Vec2::new(x0 + rx, y0));
        p.line_to(Vec2::new(x1 - rx, y0));
        p.cubic_to(
            Vec2::new(x1 - rx + kx, y0),
            Vec2::new(x1, y0 + ry - ky),
            Vec2::new(x1, y0 + ry),
        );
        p.line_to(Vec2::new(x1, y1 - ry));
        p.cubic_to(
            Vec2::new(x1, y1 - ry + ky),
            Vec2::new(x1 - rx + kx, y1),
            Vec2::new(x1 - rx, y1),
        );
        p.line_to(Vec2::new(x0 + rx, y1));
        p.cubic_to(
            Vec2::new(x0 + rx - kx, y1),
            Vec2::new(x0, y1 - ry + ky),
            Vec2::new(x0, y1 - ry),
        );
        p.line_to(Vec2::new(x0, y0 + ry));
        p.cubic_to(
            Vec2::new(x0, y0 + ry - ky),
            Vec2::new(x0 + rx - kx, y0),
            Vec2::new(x0 + rx, y0),
        );
        p.close();
        p
    }

    pub fn clear(&mut self) {
        self.verbs.clear();
        self.points.clear();
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let verb = *self.verbs.get(self.verb_idx)?;
        let n = verb.point_count();
        let seg = PathSegment {
            verb,
            points: &self.points[self.point_idx..self.point_idx + n],
        };
        self.verb_idx += 1;
        self.point_idx += n;
        Some(seg)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.verbs.len() - self.verb_idx;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for PathIter<'a> {}

#[cfg(test)]
#[path = "path_test.rs"]
mod tests;
