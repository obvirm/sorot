/// A rasterization edge — always top-to-bottom (y0 <= y1).
///
/// Horizontal edges (y0 == y1) are discarded during construction.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub x: f32,
    pub y0: i32,
    pub y1: i32,
    pub dx: f32,
    pub winding: i8,
}

impl Edge {
    #[inline]
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, winding: i8) -> Option<Self> {
        let (x0, y0, x1, y1, winding) = if y0 <= y1 {
            (x0, y0, x1, y1, winding)
        } else {
            (x1, y1, x0, y0, -winding)
        };

        let y0_floor = y0.floor() as i32;
        let y1_ceil = y1.ceil() as i32;

        if y0_floor >= y1_ceil {
            return None;
        }

        let dy = y1 - y0;
        let dx = (x1 - x0) / dy;

        let x_at_y0 = x0 + dx * (y0_floor as f32 + 0.5 - y0);

        Some(Self {
            x: x_at_y0,
            y0: y0_floor,
            y1: y1_ceil,
            dx,
            winding,
        })
    }

    #[inline]
    pub fn advance(&mut self) {
        self.x += self.dx;
        self.y0 += 1;
    }
}

pub const TILE_SIZE: u32 = 16;

#[derive(Debug, Clone)]
pub struct Tile {
    pub col: u32,
    pub row: u32,
    pub edges: Vec<Edge>,
}

impl Tile {
    pub fn new(col: u32, row: u32) -> Self {
        Self {
            col,
            row,
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileGrid {
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub tile_size: u32,
    pub cols: u32,
    pub rows: u32,
    pub tiles: Vec<Tile>,
}

impl TileGrid {
    pub fn new(pixel_width: u32, pixel_height: u32, tile_size: u32) -> Self {
        let cols = (pixel_width + tile_size - 1) / tile_size;
        let rows = (pixel_height + tile_size - 1) / tile_size;
        let mut tiles = Vec::with_capacity((cols * rows) as usize);

        for row in 0..rows {
            for col in 0..cols {
                tiles.push(Tile::new(col, row));
            }
        }

        Self {
            pixel_width,
            pixel_height,
            tile_size,
            cols,
            rows,
            tiles,
        }
    }

    #[inline]
    fn tile_index(&self, col: u32, row: u32) -> usize {
        (row * self.cols + col) as usize
    }

    #[inline]
    pub fn tile_at(&self, col: u32, row: u32) -> &Tile {
        &self.tiles[self.tile_index(col, row)]
    }

    #[inline]
    pub fn tile_at_mut(&mut self, col: u32, row: u32) -> &mut Tile {
        let idx = self.tile_index(col, row);
        &mut self.tiles[idx]
    }

    pub fn bin_edges(&mut self, edges: &[Edge]) {
        for &edge in edges {
            let y0 = edge.y0.max(0);
            let y1 = edge.y1.min(self.pixel_height as i32);

            if y0 >= y1 {
                continue;
            }

            let start_row = (y0 as u32) / self.tile_size;
            let end_row = ((y1 as u32 - 1) / self.tile_size).min(self.rows - 1);

            for row in start_row..=end_row {
                let tile_y0 = (row * self.tile_size) as i32;
                let tile_y1 = tile_y0 + self.tile_size as i32;

                if y1 <= tile_y0 || y0 >= tile_y1 {
                    continue;
                }

                let mut clipped = edge;
                clipped.y0 = y0.max(tile_y0);
                clipped.y1 = y1.min(tile_y1);

                let scanlines_to_skip = (clipped.y0 - edge.y0) as f32;
                clipped.x = edge.x + edge.dx * scanlines_to_skip;

                let x0 = (clipped.x.floor() as i32).max(0);
                let x1_at_end = edge.x + edge.dx * (clipped.y1 as f32 - edge.y0 as f32);
                let x1 = (x1_at_end.ceil() as i32).min(self.pixel_width as i32);

                if x0 >= x1 {
                    continue;
                }

                let start_col = (x0 as u32) / self.tile_size;
                let end_col = ((x1 as u32 - 1) / self.tile_size).min(self.cols - 1);

                for col in start_col..=end_col {
                    self.tile_at_mut(col, row).edges.push(clipped);
                }
            }
        }
    }

    pub fn clear_edges(&mut self) {
        for tile in &mut self.tiles {
            tile.edges.clear();
        }
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter()
    }

    pub fn iter_tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.tiles.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new_top_to_bottom() {
        let e = Edge::new(10.0, 0.0, 10.0, 10.0, 1).unwrap();
        assert_eq!(e.y0, 0);
        assert_eq!(e.y1, 10);
    }

    #[test]
    fn test_edge_new_horizontal_skipped() {
        let e = Edge::new(0.0, 5.0, 10.0, 5.0, 1);
        assert!(e.is_none());
    }

    #[test]
    fn test_edge_new_bottom_to_top_flipped() {
        let e = Edge::new(5.0, 10.0, 5.0, 0.0, 1).unwrap();
        assert!(e.y0 <= e.y1);
    }

    #[test]
    fn test_edge_advance() {
        let mut e = Edge::new(0.0, 0.0, 10.0, 10.0, 1).unwrap();
        let x_before = e.x;
        e.advance();
        assert!((e.x - x_before - 1.0).abs() < 1e-6);
        assert_eq!(e.y0, 1);
    }

    #[test]
    fn test_tile_grid_creation() {
        let grid = TileGrid::new(100, 100, 16);
        assert_eq!(grid.cols, 7);
        assert_eq!(grid.rows, 7);
        assert_eq!(grid.tiles.len(), 49);
    }

    #[test]
    fn test_bin_edges() {
        let mut grid = TileGrid::new(32, 32, 16);
        let e = Edge::new(5.0, 5.0, 25.0, 25.0, 1).unwrap();
        grid.bin_edges(&[e]);
        let total_edges: usize = grid.tiles.iter().map(|t| t.edges.len()).sum();
        assert!(total_edges > 0);
    }
}
