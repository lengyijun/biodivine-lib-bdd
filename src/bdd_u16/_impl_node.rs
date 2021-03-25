use crate::bdd_u16::{Node, NodePointer};

impl Node {
    pub fn low(&self) -> NodePointer {
        self.0
    }

    pub fn high(&self) -> NodePointer {
        self.1
    }

    /// Flip the low/high pointers in this node.
    pub fn flip(&self) -> Node {
        Node(self.1, self.0)
    }
}
