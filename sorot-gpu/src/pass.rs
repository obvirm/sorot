use sorot_render::render_ir::{PacketId, RenderFrame, SdfOp};

#[derive(Debug, Clone)]
pub enum PassKind {
    Shape {
        packet_ids: Vec<PacketId>,
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
        let mut packet_ids: Vec<PacketId> = Vec::new();
        for tile in &frame.tiles {
            for &id in &tile.packet_ids {
                if !seen[id as usize] {
                    seen[id as usize] = true;
                    packet_ids.push(id);
                }
            }
        }

        if !packet_ids.is_empty() {
            passes.push(RenderPass {
                kind: PassKind::Shape { packet_ids, target_id: shape_target },
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
