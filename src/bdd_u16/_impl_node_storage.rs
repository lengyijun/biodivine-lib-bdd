use crate::bdd_u16::{NodeStorage, VariableId, Node, NodePointer};
use std::collections::HashMap;
use fxhash::FxBuildHasher;

impl NodeStorage {

    pub fn new(capacity: usize) -> NodeStorage {
        NodeStorage {
            stats: Default::default(),
            map: HashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default())
        }
    }

    pub fn find(&mut self, variable: VariableId, node: Node) -> Option<NodePointer> {
        if node.0.is_non_trivial() && node.1.is_non_trivial() && node.0.variable_id() == node.1.variable_id() {
            self.stats.0 += 1;
        } else if node.0.is_terminal() || node.1.is_terminal() {
            self.stats.1 += 1;
        } else {
            self.stats.2 += 1;
        }
        self.map.get(&(variable, node)).cloned()
    }

    pub fn insert(&mut self, variable: VariableId, node: Node, pointer: NodePointer) {
        self.map.insert((variable, node), pointer);
    }

}