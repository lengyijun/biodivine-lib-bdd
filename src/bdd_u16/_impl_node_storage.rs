use crate::bdd_u16::{NodeStorage, VariableId, Node, NodePointer};
use std::collections::HashMap;
use fxhash::FxBuildHasher;

impl NodeStorage {

    pub fn new(capacity: usize) -> NodeStorage {
        NodeStorage { map: HashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default()) }
    }

    pub fn find(&self, variable: VariableId, node: Node) -> Option<NodePointer> {
        self.map.get(&(variable, node)).cloned()
    }

    pub fn insert(&mut self, variable: VariableId, node: Node, pointer: NodePointer) {
        self.map.insert((variable, node), pointer);
    }

}