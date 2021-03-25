use crate::bdd_u16::{Bdd, Node, NodePointer, VariableId};

impl Bdd {

    pub fn mk_false() -> Bdd {
        // Having an empty vector saves some allocations when passing around true/false Bdd object.
        // However, it means we can't further manipulate such a Bdd. Use `mk_blank` instead.
        Bdd(NodePointer::zero(), vec![])
    }

    pub fn mk_true() -> Bdd {
        // Having an empty vector saves some allocations when passing around true/false Bdd object.
        // However, it means we can't further manipulate such a Bdd. Use `mk_blank` instead.
        Bdd(NodePointer::one(), vec![])
    }

    pub fn mk_const(value: bool) -> Bdd {
        Bdd(NodePointer::terminal(value), vec![])
    }

    /// True if this `Bdd` represents a `true` formula.
    pub fn is_true(&self) -> bool {
        self.root().is_one()
    }

    /// True if this `Bdd` represents a `false` formula.
    pub fn is_false(&self) -> bool {
        self.root().is_zero()
    }

    /// Alternative to `mk_false`/`mk_true` that creates a `Bdd` that can be actually modified
    /// using `push_node`, etc. after it has been created.
    pub(super) fn mk_blank(is_true: bool) -> Bdd {
        Bdd(NodePointer::terminal(is_true), vec![vec![]; 64])
    }

    pub fn mk_var(id: VariableId, value: bool) -> Bdd {
        let mut bdd = Self::mk_blank(false);
        let node = if value {
            Node(NodePointer::zero(), NodePointer::one())
        } else {
            Node(NodePointer::one(), NodePointer::zero())
        };
        bdd.0 = bdd.push_node(id, node);
        bdd
    }

    pub(super) fn push_node(&mut self, variable: VariableId, node: Node) -> NodePointer {
        let vector = &mut self.1[usize::from(variable)];
        let node_index = vector.len();
        vector.push(node);
        NodePointer::new(variable, node_index)
    }

    pub(super) fn root(&self) -> NodePointer {
        self.0
    }

    /// Set the root pointer of this Bdd, but do so without checking if the pointer is valid!
    pub(super) fn set_root(&mut self, pointer: NodePointer) {
        self.0 = pointer;
    }

    pub(super) fn node(&self, variable: VariableId, node_index: usize) -> &Node {
        &self.1[usize::from(variable)][node_index]
    }

    /// Number of nodes that form the graph of this `Bdd`.
    pub fn node_count(&self) -> usize {
        let mut count = 0;
        for vector in &self.1 {
            count += vector.len()
        }
        count + 2 // + 2 terminal nodes
    }

}

impl Default for Bdd {
    fn default() -> Self {
        Self::mk_false()
    }
}
