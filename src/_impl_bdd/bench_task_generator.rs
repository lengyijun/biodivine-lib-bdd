use crate::{Bdd, BddPointer};
use std::cmp::{max, min};
use std::collections::HashSet;
use fxhash::FxBuildHasher;
use std::convert::TryFrom;
use crate::_impl_bdd::dynamic_op_cache::DynamicOpCache;
use crate::_impl_bdd::cache2::Cache2;
use std::option::Option::Some;

/// "Original" task generation enhanced with n-log-n initial cache size
pub fn spawn_tasks(left: &Bdd, right: &Bdd) -> usize {
    let mut stack = Vec::with_capacity(max(left.size(), right.size()));
    stack.push((left.root_pointer(), right.root_pointer()));

    let mut op_cache: HashSet<(BddPointer, BddPointer), FxBuildHasher> =
        HashSet::with_capacity_and_hasher(
            usize::try_from(n_log_n(
                    u64::try_from(left.size()).unwrap(),
                    u64::try_from(right.size()).unwrap())
            ).unwrap(),
            FxBuildHasher::default());

    while let Some(on_stack) = stack.pop() {
        if op_cache.contains(&on_stack) {
            continue;
        } else {
            op_cache.insert(on_stack.clone());
            let (l, r) = on_stack;
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

            if !op_cache.contains(&(l_low, r_low)) {
                stack.push((l_low, r_low));
            }
            if !op_cache.contains(&(l_high, r_high)) {
                stack.push((l_high, r_high));
            }
        }
    }

    op_cache.len()
}

pub(crate) fn spawn_tasks_2(left: &Bdd, right: &Bdd, op_cache: &mut Cache2, stack: &mut Vec<(BddPointer, BddPointer)>) -> (usize, usize) {
    stack.clear();
    op_cache.clear();

    stack.push((left.root_pointer(), right.root_pointer()));

    /*let capacity = usize::try_from(n_log_n(
        u64::try_from(left.size()).unwrap(),
        u64::try_from(right.size()).unwrap())
    ).unwrap();*/
    //let mut op_cache: Cache2 = Cache2::new(max(left.size(), right.size()));

    let mut i = 0;
    /*while let Some((l, r)) = stack.pop() {
        i += 1;
        if op_cache.contains(l, r) {
            continue;
        } else {
            op_cache.insert(l, r);
            let (l_v, r_v) = (left.var_of(l), right.var_of(r));
            let decision_var = min(l_v, r_v);

            let (l_low, l_high) = if l_v != decision_var {
                (l, l)
            } else {
                left.links(l)
                //(left.low_link_of(l), left.high_link_of(l))
            };
            let (r_low, r_high) = if r_v != decision_var {
                (r, r)
            } else {
                right.links(r)
                //(right.low_link_of(r), right.high_link_of(r))
            };

            if !op_cache.contains(l_low, r_low) {
                stack.push((l_low, r_low));
            }
            if !op_cache.contains(l_high, r_high) {
                stack.push((l_high, r_high));
            }
        }
    }*/
    loop {
        if stack.len() > 8 {
            i += 8;
            let x1 = stack.pop().unwrap();
            let x2 = stack.pop().unwrap();
            let x3 = stack.pop().unwrap();
            let x4 = stack.pop().unwrap();
            let x5 = stack.pop().unwrap();
            let x6 = stack.pop().unwrap();
            let x7 = stack.pop().unwrap();
            let x8 = stack.pop().unwrap();
            iteration(left, right, op_cache, stack, x1.0, x1.1);
            iteration(left, right, op_cache, stack, x2.0, x2.1);
            iteration(left, right, op_cache, stack, x3.0, x3.1);
            iteration(left, right, op_cache, stack, x4.0, x4.1);
            iteration(left, right, op_cache, stack, x5.0, x5.1);
            iteration(left, right, op_cache, stack, x6.0, x6.1);
            iteration(left, right, op_cache, stack, x7.0, x7.1);
            iteration(left, right, op_cache, stack, x8.0, x8.1);
        } else if let Some((l, r)) = stack.pop() {
            i += 1;
            iteration(left, right, op_cache, stack, l, r);
        } else {
            break;
        }
    }

    (i, op_cache.collisions)
    //(i, 0)
}

fn iteration(left: &Bdd, right: &Bdd, op_cache: &mut Cache2, stack: &mut Vec<(BddPointer, BddPointer)>, l: BddPointer, r: BddPointer) {
    if op_cache.contains(l, r) {
        return;
    } else {
        op_cache.insert(l, r);
        let (l_v, r_v) = (left.var_of(l), right.var_of(r));
        let decision_var = min(l_v, r_v);

        let (l_low, l_high) = if l_v != decision_var {
            (l, l)
        } else {
            left.links(l)
            //(left.low_link_of(l), left.high_link_of(l))
        };
        let (r_low, r_high) = if r_v != decision_var {
            (r, r)
        } else {
            right.links(r)
            //(right.low_link_of(r), right.high_link_of(r))
        };

        if !op_cache.contains(l_low, r_low) {
            stack.push((l_low, r_low));
        }
        if !op_cache.contains(l_high, r_high) {
            stack.push((l_high, r_high));
        }
    }
}

pub fn n_log_n(left: u64, right: u64) -> u64 {
    debug_assert!(left > 0);
    debug_assert!(right > 0);

    if left > right {
        left * u64::from(63u32 - right.leading_zeros())
    } else {
        right * u64::from(63u32 - left.leading_zeros())
    }
}