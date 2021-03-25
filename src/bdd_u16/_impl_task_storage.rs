use crate::bdd_u16::{TaskStorage, NodePointer};
use std::collections::HashMap;
use fxhash::FxBuildHasher;

impl TaskStorage {

    pub fn new(capacity: usize) -> TaskStorage {
        TaskStorage {
            stats: Default::default(),
            map: HashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default())
        }
    }

    pub fn is_done(&self, task: &(NodePointer, NodePointer)) -> bool {
        self.map.contains_key(task)
    }

    pub fn resolve(&mut self, left: NodePointer, right: NodePointer) -> Option<NodePointer> {
        /*if left.is_terminal() || right.is_terminal() {
            self.stats.0 += 1;
        } else {
            let left_node = left.node_index();
            let right_node = right.node_index();
            if left_node < 64 && right_node < 64 {
                self.stats.1 += 1;
            } else {
                self.stats.2 += 1;
            }
        }*/

        self.map.get(&(left, right)).cloned()
    }

    pub fn save(&mut self, left: NodePointer, right: NodePointer, result: NodePointer) {
        self.map.insert((left, right), result);
    }

}