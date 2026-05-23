use sorot_scene::render_ir::{RenderFrame, RenderPacket, SdfOp};

/// A unit of GPU work — renders to a target, optionally reads from inputs.
#[derive(Debug, Clone)]
pub enum PassKind {
    /// Render triangulated shapes to a color attachment.
    Shape {
        packets: Vec<RenderPacket>,
        target_id: usize,
    },
    /// Render an SDF glyph to a color attachment.
    Sdf {
        op: SdfOp,
        target_id: usize,
    },
    /// Composite inputs onto the swapchain (target_id = u32::MAX).
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
    pub screen_width: u32,
    pub screen_height: u32,
}

impl PassGraph {
    pub fn from_frame(
        frame: &RenderFrame,
        shape_target: usize,
        sdf_target: usize,
    ) -> Self {
        let mut passes = Vec::new();

        let all_packets: Vec<RenderPacket> = frame
            .tiles
            .iter()
            .flat_map(|t| t.packets.clone())
            .collect();

        if !all_packets.is_empty() {
            passes.push(RenderPass {
                kind: PassKind::Shape {
                    packets: all_packets,
                    target_id: shape_target,
                },
                label: "shape".into(),
            });
        }

        for op in &frame.sdf_ops {
            passes.push(RenderPass {
                kind: PassKind::Sdf {
                    op: op.clone(),
                    target_id: sdf_target,
                },
                label: "sdf".into(),
            });
        }

        let mut input_ids = Vec::new();
        if !passes.is_empty() {
            input_ids.push(shape_target);
        }
        if !frame.sdf_ops.is_empty() {
            input_ids.push(sdf_target);
        }

        if !input_ids.is_empty() {
            passes.push(RenderPass {
                kind: PassKind::Composite {
                    input_ids,
                    clear_color: [0.05, 0.05, 0.08, 1.0],
                },
                label: "composite".into(),
            });
        }

        Self {
            passes,
            screen_width: frame.screen_width,
            screen_height: frame.screen_height,
        }
    }
}
