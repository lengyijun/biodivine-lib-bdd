use crate::bdd_u16::{TaskStorage, NodePointer};
use std::collections::HashMap;
use fxhash::FxBuildHasher;

impl TaskStorage {

    pub fn new(capacity: usize) -> TaskStorage {
        TaskStorage { map: HashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default()) }
    }

    pub fn is_done(&self, task: &(NodePointer, NodePointer)) -> bool {
        self.map.contains_key(task)
    }

    pub fn resolve(&self, left: NodePointer, right: NodePointer) -> Option<NodePointer> {
        self.map.get(&(left, right)).cloned()
    }

    pub fn save(&mut self, left: NodePointer, right: NodePointer, result: NodePointer) {
        self.map.insert((left, right), result);
    }

}