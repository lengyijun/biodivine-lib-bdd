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

    /// Alternative to `mk_false`/`mk_true` that creates a `Bdd` that can be actually modified
    /// using `push_node`, etc. after it has been created.
    pub(super) fn mk_blank(is_true: bool) -> Bdd {
        Bdd(NodePointer::terminal(is_true), vec![vec![]; 64])
    }

    pub(super) fn mk_var(id: VariableId, value: bool) -> Bdd {
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

}

impl Default for Bdd {
    fn default() -> Self {
        Self::mk_false()
    }
}
