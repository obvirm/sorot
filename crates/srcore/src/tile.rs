use slotmap::SlotMap;
use srmath::BoundingBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

pub const TILE_SIZE: u32 = 32;

impl TileCoord {
    pub fn from_pixel(px: f32, py: f32) -> Self {
        Self {
            x: (px / TILE_SIZE as f32).floor() as i32,
            y: (py / TILE_SIZE as f32).floor() as i32,
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let x = self.x as f32 * TILE_SIZE as f32;
        let y = self.y as f32 * TILE_SIZE as f32;
        BoundingBox::new(
            srmath::Vec2::new(x, y),
            srmath::Vec2::new(x + TILE_SIZE as f32, y + TILE_SIZE as f32),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub coord: TileCoord,
    pub shape_ids: Vec<usize>,
    pub dirty: bool,
}

impl Tile {
    pub fn new(coord: TileCoord) -> Self {
        Self {
            coord,
            shape_ids: Vec::new(),
            dirty: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileMap {
    pub tiles: SlotMap<slotmap::DefaultKey, Tile>,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl TileMap {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let tiles_x = (screen_width as f32 / TILE_SIZE as f32).ceil() as i32;
        let tiles_y = (screen_height as f32 / TILE_SIZE as f32).ceil() as i32;

        let mut tiles = SlotMap::with_key();
        for y in 0..tiles_y {
            for x in 0..tiles_x {
                tiles.insert(Tile::new(TileCoord { x, y }));
            }
        }

        Self {
            tiles,
            screen_width,
            screen_height,
        }
    }

    pub fn tiles_for_bounds(&self, min: TileCoord, max: TileCoord) -> Vec<TileCoord> {
        let mut coords = Vec::new();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                coords.push(TileCoord { x, y });
            }
        }
        coords
    }

    pub fn clear_dirty(&mut self) {
        for tile in self.tiles.values_mut() {
            tile.shape_ids.clear();
            tile.dirty = false;
        }
    }
}
