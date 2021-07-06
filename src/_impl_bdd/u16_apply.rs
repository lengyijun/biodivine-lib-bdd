use crate::{Bdd, BddPointer, BddNode};
use std::convert::TryFrom;
use std::cmp::{min, max};
use std::collections::HashMap;
use fxhash::FxBuildHasher;

/// A stack-allocated cache for completed Bdd tasks.
///
/// If stores 2-byte pointers, so it cannot address more than 2^16 - 1 Bdd nodes.
struct StaticOpCache<const X: usize> {
    l_size: usize, r_size: usize,
    storage: [u16; X]
}

struct StaticTaskStack {
    after_top: usize,
    storage: [(BddPointer, BddPointer); 1024]
}

impl StaticTaskStack {
    pub fn new(num_vars: u16) -> StaticTaskStack {
        if num_vars*2 > 1024 {  // This should more-or-less hold, but please prove it...
            panic!("Possible static stack overflow.");
        }
        StaticTaskStack {
            after_top: 0,
            storage: [(BddPointer::zero(), BddPointer::zero()); 1024],
        }
    }

    pub fn push(&mut self, task: (BddPointer, BddPointer)) {
        if self.after_top >= 1024 {
            panic!("Static stack overflow.");
        }
        self.storage[self.after_top] = task;
        self.after_top += 1;
    }

    pub fn pop(&mut self) -> Option<(BddPointer, BddPointer)> {
        if self.after_top == 0 {
            None
        } else {
            self.after_top -= 1;
            Some(self.storage[self.after_top])
        }
    }
}

impl <const X: usize> StaticOpCache<X> {

    pub fn new(left: &Bdd, right: &Bdd) -> StaticOpCache<X> {
        if left.size() * right.size() > X {
            panic!("Invalid OpCache: worst case size exceeds cache size.");
        }
        if left.size() * right.size() >= usize::from(u16::MAX) {
            panic!("Invalid OpCache: worst case size exceeds 16-bit address space.")
        }
        StaticOpCache {
            l_size: left.size(),
            r_size: right.size(),
            storage: [u16::MAX; X]
        }
    }

    pub fn get(&self, l_pointer: BddPointer, r_pointer: BddPointer) -> Option<BddPointer> {
        let index = self.index(l_pointer, r_pointer);
        match self.storage[index] {
            u16::MAX => None,
            x => Some(BddPointer(u32::from(x)))
        }
    }

    pub fn contains(&self, l_pointer: BddPointer, r_pointer: BddPointer) -> bool {
        let index = self.index(l_pointer, r_pointer);
        self.storage[index] != u16::MAX
    }

    pub fn set(&mut self, l_pointer: BddPointer, r_pointer: BddPointer, value: BddPointer) {
        let index = self.index(l_pointer, r_pointer);
        self.storage[index] = u16::try_from(value.0).unwrap();
    }

    fn index(&self, l_pointer: BddPointer, r_pointer: BddPointer) -> usize {
        usize::try_from(l_pointer.0 * u32::try_from(self.r_size).unwrap() + r_pointer.0).unwrap()
    }

}

pub fn apply<T>(left: &Bdd, right: &Bdd, terminal_lookup: T) -> Bdd where
    T: Fn(Option<bool>, Option<bool>) -> Option<bool>, {
    let worst_case_size = left.size() * right.size();
    if worst_case_size < 1024 {
        apply_fixed(left, right, terminal_lookup, StaticOpCache::<1024>::new(left, right))
    } else if worst_case_size < 65535 { // u16::MAX
        apply_fixed(left, right, terminal_lookup, StaticOpCache::<65535>::new(left, right))
    } else {
        panic!("Cannot apply to this bdd size.");
    }
}

fn apply_fixed<T, const X: usize>(
    left: &Bdd,
    right: &Bdd,
    terminal_lookup: T,
    mut op_cache: StaticOpCache<X>) -> Bdd where
    T: Fn(Option<bool>, Option<bool>) -> Option<bool>, {

    let mut is_empty = true;

    /*let mut existing: HashMap<BddNode, BddPointer, FxBuildHasher> =
        HashMap::with_capacity_and_hasher(max(left.size(), right.size()), FxBuildHasher::default());
    existing.insert(BddNode::mk_zero(left.num_vars()), BddPointer::zero());
    existing.insert(BddNode::mk_one(left.num_vars()), BddPointer::one());*/

    let mut result = Bdd::mk_true(left.num_vars());
    Extend::<BddNode>::extend_reserve(&mut result.0, max(left.size(), right.size()));

    let mut stack = StaticTaskStack::new(left.num_vars());
    stack.push((left.root_pointer(), right.root_pointer()));

    while let Some((l, r)) = stack.pop() {
        if op_cache.contains(l, r) {
           continue;    // Task already done.
        }

        let (l_v, r_v) = (left.var_of(l), right.var_of(r));
        let decision_var = min(l_v, r_v);

        let (l_low, l_high) = if l_v != decision_var {
            (l, l)
        } else {
            (left.low_link_of(l), left.high_link_of(l))
        };
        let (r_low, r_high) = if r_v != decision_var {
            (r, r)
        } else {
            (right.low_link_of(r), right.high_link_of(r))
        };

        // Try to solve the tasks using terminal lookup table or from cache.
        let new_low = terminal_lookup(l_low.as_bool(), r_low.as_bool())
            .map(BddPointer::from_bool)
            .or_else(|| op_cache.get(l_low, r_low));
        let new_high = terminal_lookup(l_high.as_bool(), r_high.as_bool())
            .map(BddPointer::from_bool)
            .or_else(|| op_cache.get(l_high, r_high));

        if let Some((new_low, new_high)) = new_low.zip(new_high) {
            if new_low.is_one() || new_high.is_one() {
                is_empty = false;
            }

            if new_low == new_high {
                op_cache.set(l, r, new_low);
            } else {
                let node = BddNode::mk_node(decision_var, new_low, new_high);
                result.push_node(node);
                op_cache.set(l, r, result.root_pointer());
            }
        } else {
            stack.push((l, r));
            if new_low.is_none() {
                stack.push((l_low, r_low));
            }
            if new_high.is_none() {
                stack.push((l_high, r_high));
            }
        }
    }


    if is_empty {
        Bdd::mk_false(left.num_vars())
    } else {
        result
    }
}

impl Bdd {
    fn minify(self) -> Bdd {
        if self.is_false() {
            return self;
        }

        let mut nodes: Vec<_> = self.0.into_iter().enumerate().collect();
        nodes.sort_unstable_by(|(_, n_a), (_, n_b)| {
            if n_a.var != n_b.var {
                n_a.var.cmp(&n_b.var).reverse()
            } else if n_a.low_link != n_b.low_link {
                n_a.low_link.cmp(&n_b.low_link)
            } else {
                n_a.high_link.cmp(&n_b.high_link)
            }
        });

        assert!(nodes[0].1.is_zero());
        assert!(nodes[1].1.is_one());

        let mut new_index = Vec::with_capacity(nodes.len());
        new_index.push(0);
        new_index.push(1);

        let mut last_node = nodes[1];
        for (i, n) in nodes.iter().skip(2) {
            if *n == last_node.1 {   // duplicate node
                new_index.push(last_node.0);
            } else {
                new_index.push(*i);
            }
        }

        unimplemented!()
    }
}