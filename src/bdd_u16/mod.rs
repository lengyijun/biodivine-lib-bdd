//! A new alternative minimalistic implementation of `Bdds` that we can use for testing
//! and experiments in hashing and algorithms in general.

use std::collections::HashMap;
use fxhash::FxBuildHasher;
use crate::BddPointer;

mod _impl_bdd;
mod _impl_bdd_apply;
mod _impl_node;
mod _impl_node_pointer;
mod _impl_variable_id;
mod _impl_node_storage;
mod _impl_task_storage;

/// Node pointer identifies one node in a `Bdd`. It actually packs two pieces of information
/// together: the variable id and the pointer to that variables' node vector. The variable
/// id is encoded using a variable length encoding. Variables are distributed in blocks of
/// 8 (0b000-0b110), where each block of 8 variables is divided into a "low block" (0b000-b0011)
/// and "high block" (0b001-0b111).
///
/// The structure of the pointer is then as follows (least significant bits are on the right): first,
/// a certain number of zeroes encodes the ID of the 8-block followed by one. The next three bits
/// are the variable id inside that block, and finally, the remaining bits are the bits of the
/// node id inside that variables storage. Consider the pointer `pppp_pppp_pvvv_1000`. Here, we
/// have the 4-th variable 8-block (3 zeroes - first block has no zero)., Then `vvv` is the
/// id of the variable inside that block, and finally, `ppppppppp` is the id of the node.
/// Note that we use this order because it is easy to extract individual information quickly using
/// shifting. If the order was reversed, the operation would be still relatively easy to obtain,
/// but the operations would have to be reversed. This is not exactly a performance bottleneck,
/// but since it complicates both implementation and the actual assembly code, we just use the
/// slightly less intuitive variant. (In favour of the second, more intuitive variant plays that
/// it works better with variable-width pointers where we could theoretically switch between
/// u16/u32/u64 based on the number of nodes)
///
/// This type of scheme allows us to assign more address space to "middle" variables in the Bdd
/// which will most likely need it much more then the very low or very high variables. For 0 and
/// 1 pointer, the literal zero still ok, and one becomes the `1000_0000_0000_0000` literal.
///
/// Finally, there at the moment, we set the smallest "reasonable" node pointer width to 4 bits.
/// Together with 3 bits per block and one marker bit, this leaves 8 blocks with 8 variables,
/// so 64 variables in total (and 2^12 nodes limit for the middle variables). However,
/// the `BddVariableSet` should prioritise allocation of the "middle" variables with more
/// address space in order to avoid address availability problems.
///
/// In layman terms, if we only have 8 variables, they all should be allocated in the 2^12 address
/// range. If we have 24 variables, 8 should have 2^12 addresses, 8 have 2^11 and 8 have 2^10.
///
/// This also leaves a small range of pointers (2^6+2^5+2^4+2^3+2^2=124; last two are the
/// reserved 0/1) that cannot be allocated. This address space can be used for example by
/// data structures that need to store pointers to represent special values (like a missing
/// value).
///
/// Furthermore, this representation allows us to easily test if two node pointers point to the
/// same variable node array, as this seems to be a common pattern in Bdd algorithms which we
/// may exploit to improve performance of some "hot paths".
///
/// Note that in the future, we may want to change the balance slightly. We might even want to
/// alternate the number of variable bits in a block depending on the bit-width of the pointer.
/// For 16 bits, 8*8 variables is reasonable. For 32 bits, 8*24=192 is not as compelling, 32*22
/// or 64*21 certainly seems more interesting. However, we will also have to consider other
/// factors. For example, in u32, this is not as bad, but as we move to something like u64, we
/// might want much wider middle levels, as addressing 2^56 nodes *per level* certainly isn't
/// realistic any time soon. In particular going beyond u64 should be pointless - that address
/// space should realistically handle any Bdd that a 64-bit processor can handle.
///
/// Bdd pointers are internal data structures and should never be exposed to code outside of this
/// crate. We can therefore allow them to be fairly "insecure" in terms of how they are passed
/// around (and transformed from numbers to pointers and back), because we can guarantee they
/// are never going to leave the context of a particular Bdd.
///
// TODO: Do we want some kind of order on bdd node pointers? I guess we can have it, but is it necessary?
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct NodePointer(u16);

/// For variable IDs, we use u32 for two pragmatic reasons. First, even in a very wide pointer
/// implementation (say, 128 bits, which would be , we are
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VariableId(pub u32);

/// A `Bdd` node consists of just two `NodePointers` (low and high). Note that the `VariableId`
/// is not part of the node, but is instead inferred from context (i.e. from the `NodePointer`
/// that defines this particular node).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Node(NodePointer, NodePointer);

/// A Bdd is a heap-allocated array of X vectors (one per variable), together with a
/// pointer to the root `Node`.
///
/// We generally assume that a `Bdd` is minimal - i.e. all `Nodes` are reachable from the root
/// node, and the order of nodes in the vectors is predictable (DFS post order from the
/// root node). We can therefore test `Bdd` equality simply by comparing the values directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bdd(NodePointer, Vec<Vec<Node>>);

struct NodeStorage {
    stats: (u64, u64, u64),
    map: HashMap<(VariableId, Node), NodePointer, FxBuildHasher>
}

struct TaskStorage {
    stats: (u64, u64, u64),
    map: HashMap<(NodePointer, NodePointer), NodePointer, FxBuildHasher>
}

struct NewNodeStorage {
    vars: Vec<VarNodeStorage>
}

struct VarNodeStorage {
    constant: [NodePointer; 4],
    terminal: [Vec<NodePointer>; 4],
    equal_vars: Vec<Vec<NodePointer>>,
    other: HashMap<(NodePointer, NodePointer), NodePointer>
}

impl NewNodeStorage {

    pub fn new(var_count: usize, _capacity: usize) -> NewNodeStorage {
        let mut vars = vec![];
        for _ in 0..var_count {
            vars.push(VarNodeStorage::new(var_count))
        }
        NewNodeStorage { vars }

    }

    pub fn find(&mut self, variable: VariableId, node: Node) -> Option<NodePointer> {
        self.vars[usize::from(variable)].find(node.0, node.1)
    }

    pub fn insert(&mut self, variable: VariableId, node: Node, pointer: NodePointer) {
        self.vars[usize::from(variable)].insert(node.0, node.1, pointer);
    }

}

impl VarNodeStorage {

    pub fn new(vars: usize) -> VarNodeStorage {
        let mut equal_vars = Vec::new();
        for _ in 0..vars {
            equal_vars.push(Vec::with_capacity(64));
        }
        VarNodeStorage {
            constant: [NodePointer::none_pointer(); 4],
            terminal: [Vec::with_capacity(64), Vec::with_capacity(64), Vec::with_capacity(64), Vec::with_capacity(64)],
            equal_vars,
            other: HashMap::new()
        }
    }

    pub fn find(&self, low: NodePointer, high: NodePointer) -> Option<NodePointer> {
        let value: Option<NodePointer> = match (low.as_bool(), high.as_bool()) {
            (Some(false), Some(false)) => Some(self.constant[0]),
            (Some(false), Some(true)) => Some(self.constant[1]),
            (Some(true), Some(false)) => Some(self.constant[2]),
            (Some(true), Some(true)) => Some(self.constant[3]),
            (Some(false), _) => self.terminal[0].get(high.node_index()).cloned(),
            (Some(true), _) => self.terminal[1].get(high.node_index()).cloned(),
            (_, Some(false)) => self.terminal[2].get(low.node_index()).cloned(),
            (_, Some(true)) => self.terminal[3].get(low.node_index()).cloned(),
            (None, None) => {
                if low.variable_id() != high.variable_id() {
                    self.other.get(&(low, high)).cloned()
                } else {
                    let vector = &self.equal_vars[usize::from(low.variable_id())];
                    let index_low = low.node_index();
                    let index_high = high.node_index();
                    let index = interleave(index_low as u64, index_high as u64) as usize;
                    vector.get(index).cloned()
                }
            }
        };

        value.and_then(|p| p.as_pointer())
    }

    pub fn insert(&mut self, low: NodePointer, high: NodePointer, result: NodePointer) {
        match (low.as_bool(), high.as_bool()) {
            (Some(false), Some(false)) => self.constant[0] = result,
            (Some(false), Some(true)) => self.constant[1] = result,
            (Some(true), Some(false)) => self.constant[2] = result,
            (Some(true), Some(true)) => self.constant[3] = result,
            (Some(false), _) => vec_insert(&mut self.terminal[0], high.node_index(), result),
            (Some(true), _) => vec_insert(&mut self.terminal[1], high.node_index(), result),
            (_, Some(false)) => vec_insert(&mut self.terminal[2], low.node_index(), result),
            (_, Some(true)) => vec_insert(&mut self.terminal[3], low.node_index(), result),
            (None, None) => {
                if low.variable_id() != high.variable_id() {
                    self.other.get(&(low, high)).cloned();
                } else {
                    let vector = &mut self.equal_vars[usize::from(low.variable_id())];
                    let index_low = low.node_index();
                    let index_high = high.node_index();
                    let index = interleave(index_low as u64, index_high as u64) as usize;
                    vec_insert(vector, index, result);
                }
            }
        };
    }

}

fn interleave(b1: u64, b2: u64) -> u64 {
    (((b2 * 0x0101010101010101 & 0x8040201008040201) *
        0x0102040810204081 >> 49) & 0x5555) |
        (((b1 * 0x0101010101010101 & 0x8040201008040201) *
            0x0102040810204081 >> 48) & 0xAAAA)
}

fn vec_insert(vector: &mut Vec<NodePointer>, index: usize, pointer: NodePointer) {
    if index >= vector.len() {
        let reserve = index - vector.len() + 1;
        vector.reserve(reserve);
        for _ in 0..reserve {
            vector.push(NodePointer::none_pointer())
        }
    }
    vector[index] = pointer;
}

/*
/// Pointer map is a mapping from (NodePointer, NodePointer) to a single NodePointer. As such,
/// it can be used for node uniqueness decisions (assuming we have one pointer map for each
/// variable), but also as an operation cache, where the two pointers are from two different
/// Bdd objects and the result is a pointer in the newly created Bdd.
///
/// It relies on several assumptions for efficient use of resources:
///  1. Queries for (1, pointer), (0, pointer), (pointer, 1) and (pointer, 0) will be common.
///  2. Queries where var(left) == var(right) will be common.
///
/// For these, a more efficient and collision resistant tree-structure is constructed. Remaining
/// values are simply delegated to a HashMap.
struct PointerMap {
    //TODO
}
*/