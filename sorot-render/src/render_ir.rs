use sorot_compositor::blend::BlendMode;
use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;

/// Vertex with premultiplied color for GPU upload.
#[derive(Debug, Clone, Copy)]
pub struct GpuVertex {
    pub clip_x: f32,
    pub clip_y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub type PacketId = u32;

/// A batch of triangles sharing one paint + transform.
#[derive(Debug, Clone)]
pub struct RenderPacket {
    pub vertices: Box<[GpuVertex]>,
    pub indices: Box<[u32]>,
    pub paint: Paint,
    pub transform: Matrix3x2,
    pub clip_rect: Rect,
}

/// A screen-space tile referencing packets by index.
#[derive(Debug, Clone)]
pub struct TilePacket {
    pub col: u32,
    pub row: u32,
    pub rect: Rect,
    pub packet_ids: Vec<PacketId>,
}

/// SDF glyph upload operation.
#[derive(Debug, Clone)]
pub struct SdfOp {
    pub sdf_data: Box<[u8]>,
    pub sdf_width: u32,
    pub sdf_height: u32,
    pub bounds: Rect,
    pub paint: Paint,
}

/// Layer compositing operation.
#[derive(Debug, Clone)]
pub struct CompositeOp {
    pub src_rect: Rect,
    pub dst_rect: Rect,
    pub blend_mode: BlendMode,
    pub opacity: f32,
}

/// The complete render IR for a frame.
#[derive(Debug, Clone)]
pub struct RenderFrame {
    pub packet_arena: Vec<RenderPacket>,
    pub tiles: Vec<TilePacket>,
    pub sdf_ops: Vec<SdfOp>,
    pub composite_ops: Vec<CompositeOp>,
    pub screen_width: u32,
    pub screen_height: u32,
    pub tile_size: u32,
}

impl RenderFrame {
    pub fn new(screen_width: u32, screen_height: u32, tile_size: u32) -> Self {
        let cols = (screen_width + tile_size - 1) / tile_size;
        let rows = (screen_height + tile_size - 1) / tile_size;
        let mut tiles = Vec::with_capacity((cols * rows) as usize);

        for row in 0..rows {
            for col in 0..cols {
                tiles.push(TilePacket {
                    col,
                    row,
                    rect: Rect::new(
                        Vec2::new((col * tile_size) as f32, (row * tile_size) as f32),
                        Vec2::new(
                            ((col + 1) * tile_size).min(screen_width) as f32,
                            ((row + 1) * tile_size).min(screen_height) as f32,
                        ),
                    ),
                    packet_ids: Vec::new(),
                });
            }
        }

        Self {
            packet_arena: Vec::new(),
            tiles,
            sdf_ops: Vec::new(),
            composite_ops: Vec::new(),
            screen_width,
            screen_height,
            tile_size,
        }
    }

    pub fn cols(&self) -> u32 {
        (self.screen_width + self.tile_size - 1) / self.tile_size
    }

    pub fn rows(&self) -> u32 {
        (self.screen_height + self.tile_size - 1) / self.tile_size
    }

    pub fn tile_at_mut(&mut self, col: u32, row: u32) -> &mut TilePacket {
        let idx = (row * self.cols() + col) as usize;
        &mut self.tiles[idx]
    }

    /// Insert a packet into the arena and bin its ID into overlapping tiles.
    pub fn bin_packet(&mut self, packet: RenderPacket) {
        let id = self.packet_arena.len() as PacketId;
        let clip = packet.clip_rect;
        self.packet_arena.push(packet);

        let ts = self.tile_size as f32;
        let start_col = (clip.min.x.max(0.0) / ts).floor() as u32;
        let start_row = (clip.min.y.max(0.0) / ts).floor() as u32;
        let end_col = ((clip.max.x.min(self.screen_width as f32) / ts).ceil() as u32).min(self.cols());
        let end_row = ((clip.max.y.min(self.screen_height as f32) / ts).ceil() as u32).min(self.rows());

        for row in start_row..end_row {
            for col in start_col..end_col {
                self.tile_at_mut(col, row).packet_ids.push(id);
            }
        }
    }

    pub fn get_packet(&self, id: PacketId) -> &RenderPacket {
        &self.packet_arena[id as usize]
    }

    pub fn non_empty_tiles(&self) -> usize {
        self.tiles.iter().filter(|t| !t.packet_ids.is_empty()).count()
    }
}
