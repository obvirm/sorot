use sorot_render::render_ir::{PacketId, RenderFrame, SdfOp};

/// A tile-row group: all packets for tiles in one row, deduplicated.
/// This preserves tile locality through the pass graph so the GPU
/// sees consecutive draws from the same screen region.
#[derive(Debug, Clone)]
pub struct TileRowGroup {
    pub row: u32,
    pub packet_ids: Vec<PacketId>,
}

#[derive(Debug, Clone)]
pub enum PassKind {
    Shape {
        tile_groups: Vec<TileRowGroup>,
        target_id: usize,
    },
    Sdf {
        op: SdfOp,
        target_id: usize,
    },
    Composite {
        input_ids: Vec<usize>,
        clear_color: [f64; 4],
    },
}

#[derive(Debug, Clone)]
pub struct RenderPass {
    pub kind: PassKind,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct PassGraph {
    pub passes: Vec<RenderPass>,
}

impl PassGraph {
    pub fn from_frame(
        frame: &RenderFrame,
        shape_target: usize,
        sdf_target: usize,
    ) -> Self {
        let mut passes = Vec::new();
        let mut seen = vec![false; frame.packet_arena.len()];
        let rows = frame.rows();
        let mut tile_groups: Vec<TileRowGroup> = Vec::with_capacity(rows as usize);

        for row in 0..rows {
            let mut row_ids = Vec::new();
            for col in 0..frame.cols() {
                let idx = (row * frame.cols() + col) as usize;
                if idx < frame.tiles.len() {
                    for &pid in &frame.tiles[idx].packet_ids {
                        if !seen[pid as usize] {
                            seen[pid as usize] = true;
                            row_ids.push(pid);
                        }
                    }
                }
            }
            if !row_ids.is_empty() {
                tile_groups.push(TileRowGroup { row, packet_ids: row_ids });
            }
        }

        if !tile_groups.is_empty() {
            passes.push(RenderPass {
                kind: PassKind::Shape { tile_groups, target_id: shape_target },
                label: "shape".into(),
            });
        }

        for op in &frame.sdf_ops {
            passes.push(RenderPass {
                kind: PassKind::Sdf { op: op.clone(), target_id: sdf_target },
                label: "sdf".into(),
            });
        }

        let has_shape = !passes.is_empty();
        let has_sdf = !frame.sdf_ops.is_empty();
        if has_shape || has_sdf {
            let mut input_ids = Vec::new();
            if has_shape { input_ids.push(shape_target); }
            if has_sdf { input_ids.push(sdf_target); }
            passes.push(RenderPass {
                kind: PassKind::Composite { input_ids, clear_color: [0.05, 0.05, 0.08, 1.0] },
                label: "composite".into(),
            });
        }

        Self { passes }
    }
}
