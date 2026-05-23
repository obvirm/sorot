use slotmap::{new_key_type, SlotMap};
use srmath::{Transform, Vec2};

new_key_type! {
    pub struct NodeId;
}

pub struct SceneNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub transform: Transform,
    pub visible: bool,
}

pub struct Scene {
    nodes: SlotMap<NodeId, SceneNode>,
    root: NodeId,
}

impl Scene {
    pub fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(SceneNode {
            id: NodeId::default(),
            parent: None,
            children: Vec::new(),
            transform: Transform::IDENTITY,
            visible: true,
        });
        Self { nodes, root }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn create_node(&mut self, parent: NodeId, transform: Transform) -> NodeId {
        let id = self.nodes.insert(SceneNode {
            id: NodeId::default(),
            parent: Some(parent),
            children: Vec::new(),
            transform,
            visible: true,
        });
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.children.push(id);
        }
        self.nodes[id].id = id;
        id
    }

    pub fn get(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id)
    }

    pub fn world_transform(&self, id: NodeId) -> Transform {
        let mut transform = Transform::IDENTITY;
        let mut current = Some(id);
        while let Some(node_id) = current {
            if let Some(node) = self.nodes.get(node_id) {
                transform = Transform {
                    translation: transform.translation + node.transform.translation,
                    rotation: transform.rotation + node.transform.rotation,
                    scale: Vec2::new(
                        transform.scale.x * node.transform.scale.x,
                        transform.scale.y * node.transform.scale.y,
                    ),
                };
                current = node.parent;
            } else {
                break;
            }
        }
        transform
    }

    pub fn iter(&self) -> impl Iterator<Item = &SceneNode> {
        self.nodes.values()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
