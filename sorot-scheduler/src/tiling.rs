use sorot_core::math::Rect;

use sorot_raster::tile::{Tile, TileGrid};

use crate::worker::{TilePriority, WorkerPool};

#[derive(Debug, Clone)]
pub struct TileJob {
    pub tile_col: u32,
    pub tile_row: u32,
    pub clip_rect: Rect,
    pub priority: TilePriority,
}

pub struct TileScheduler {
    pool: WorkerPool,
}

impl TileScheduler {
    pub fn new() -> Self {
        Self {
            pool: WorkerPool::default(),
        }
    }

    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            pool: WorkerPool::new(num_threads),
        }
    }

    /// Classify tiles by priority based on viewport intersection.
    pub fn classify_tiles(
        grid: &TileGrid,
        viewport: Rect,
    ) -> (Vec<TileJob>, Vec<TileJob>, Vec<TileJob>) {
        let mut visible = Vec::new();
        let mut foreground = Vec::new();
        let mut background = Vec::new();

        for tile in grid.iter_tiles() {
            let tile_rect = tile_to_rect(tile, grid.tile_size);

            let job = TileJob {
                tile_col: tile.col,
                tile_row: tile.row,
                clip_rect: tile_rect,
                priority: TilePriority::Visible,
            };

            if tile_rect.intersects(viewport) {
                visible.push(job);
            } else if tile_rect.min.x < viewport.max.x + grid.tile_size as f32 * 2.0
                && tile_rect.max.x > viewport.min.x - grid.tile_size as f32 * 2.0
                && tile_rect.min.y < viewport.max.y + grid.tile_size as f32 * 2.0
                && tile_rect.max.y > viewport.min.y - grid.tile_size as f32 * 2.0
            {
                let mut j = job;
                j.priority = TilePriority::Foreground;
                foreground.push(j);
            } else {
                let mut j = job;
                j.priority = TilePriority::Background;
                background.push(j);
            }
        }

        (visible, foreground, background)
    }

    /// Rasterize tiles in parallel with priority ordering.
    pub fn rasterize_tiles<F>(&self, grid: &TileGrid, viewport: Rect, rasterize: F)
    where
        F: Fn(&Tile, &TileJob) + Send + Sync,
    {
        let (visible, foreground, background) = Self::classify_tiles(grid, viewport);

        self.pool.schedule_priority(
            &visible,
            &foreground,
            &background,
            move |job, _prio| {
                if let Some(tile) = grid_tile_by_coords(grid, job.tile_col, job.tile_row) {
                    rasterize(tile, job);
                }
            },
        );
    }
}

impl Default for TileScheduler {
    fn default() -> Self {
        Self::new()
    }
}

fn tile_to_rect(tile: &Tile, tile_size: u32) -> Rect {
    let min_x = (tile.col * tile_size) as f32;
    let min_y = (tile.row * tile_size) as f32;
    Rect::new(
        sorot_core::math::Vec2::new(min_x, min_y),
        sorot_core::math::Vec2::new(min_x + tile_size as f32, min_y + tile_size as f32),
    )
}

fn grid_tile_by_coords(grid: &TileGrid, col: u32, row: u32) -> Option<&Tile> {
    if col < grid.cols && row < grid.rows {
        Some(grid.tile_at(col, row))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sorot_core::math::Vec2;

    #[test]
    fn test_classify_tiles() {
        let grid = TileGrid::new(64, 64, 16);
        let viewport = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0));

        let (v, f, b) = TileScheduler::classify_tiles(&grid, viewport);
        assert!(!v.is_empty());
        assert!(v.len() + f.len() + b.len() == grid.tiles.len());
    }
}
