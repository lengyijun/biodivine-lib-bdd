use crate::bdd_u16::{Bdd, NodeStorage, TaskStorage, NodePointer, Node, NewNodeStorage};
use std::option::Option::Some;
use std::cmp::{min, max};

impl Bdd {

    pub fn not(&self) -> Bdd {
        if self.is_true() {
            Bdd::mk_false()
        } else if self.is_false() {
            Bdd::mk_true()
        } else {
            // Note that this does not break DFS order of the graph because
            // we are only flipping terminals, which already have special positions.
            let mut result_vector = self.1.clone();
            for vector in result_vector.iter_mut() {
                for node in vector.iter_mut() {
                    node.0 = node.0.flip_if_terminal();
                    node.1 = node.1.flip_if_terminal();
                }
            }
            Bdd(self.root(), result_vector)
        }
    }

    /// Create a `Bdd` corresponding to the $\phi \land \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn and(&self, right: &Bdd) -> Bdd {
        apply(self, right, crate::op_function::and)
    }

    /// Create a `Bdd` corresponding to the $\phi \lor \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn or(&self, right: &Bdd) -> Bdd {
        apply(self, right, crate::op_function::or)
    }

    /// Create a `Bdd` corresponding to the $\phi \Rightarrow \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn imp(&self, right: &Bdd) -> Bdd {
        apply(self, right, crate::op_function::imp)
    }

    /// Create a `Bdd` corresponding to the $\phi \Leftrightarrow \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn iff(&self, right: &Bdd) -> Bdd {
        apply(self, right, crate::op_function::iff)
    }

    /// Create a `Bdd` corresponding to the $\phi \oplus \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn xor(&self, right: &Bdd) -> Bdd {
        apply(self, right, crate::op_function::xor)
    }

    /// Create a `Bdd` corresponding to the $\phi \land \neg \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn and_not(&self, right: &Bdd) -> Bdd {
        apply(self, right, crate::op_function::and_not)
    }

}


pub(super) fn apply<T>(
    left: &Bdd,
    right: &Bdd,
    lookup_table: T
) -> Bdd
where
    T: Fn(Option<bool>, Option<bool>) -> Option<bool>
{
    println!("Apply: {} {}", left.node_count(), right.node_count());
    // If the arguments are trivial, we may be able to resolve them using lookup table only:
    let left_const = left.root().as_bool();
    let right_const = right.root().as_bool();
    if let Some(result) = lookup_table(left_const, right_const) {
        return Bdd::mk_const(result);
    }

    if let (Some(l), Some(r)) = (left_const, right_const) {
        panic!("Lookup table error. Unable to resolve constant nodes ({}, {}).", l, r);
    }

    // At this point, we can assume at least one of the roots is non-trivial.

    let mut output = Bdd::mk_blank(false);

    let capacity = max(left.node_count(), right.node_count());
    //let mut nodes = NewNodeStorage::new(max(left.1.len(), right.1.len()), capacity); //NodeStorage::new(capacity);
    let mut nodes = NodeStorage::new(capacity);
    let mut tasks = TaskStorage::new(capacity);

    let mut task_stack: Vec<(NodePointer, NodePointer)> = Vec::new();
    task_stack.push((left.root(), right.root()));

    while let Some(last) = task_stack.last() {
        if tasks.resolve(last.0, last.1).is_some() {
            task_stack.pop();
        } else {
            let (l, r) = (last.0, last.1);

            // Determine which variable we are conditioning on, assuming smallest variables are
            // in the top layers of the Bdd.
            let l_var = if l.is_terminal() { None } else { Some(l.variable_id()) };
            let r_var = if r.is_terminal() { None } else { Some(r.variable_id()) };
            let condition_var = match (l_var, r_var) {
                (Some(x), Some(y)) => min(x,y),
                (Some(v), None) => v,
                (None, Some(v)) => v,
                (None, None) => {
                    // If this happens, it means the task was not resolved by lookup table
                    // and was instead pushed to stack and came here. Which should not happen
                    // because lookup table needs to resolve every terminal pair.
                    panic!("Lookup table error. Unable to resolve constant nodes ({}, {}).", l.as_bool().unwrap(), r.as_bool().unwrap());
                }
            };

            // Now, determine the left and right children that the result for (l, r) depends on.
            // This boils down to advancing the pointer if condition_var is the same as variable_id
            // of that pointer.
            let (l_low, l_high) = if Some(condition_var) == l_var {
                let l_node = left.node(condition_var, l.node_index());
                (l_node.low(), l_node.high())
            } else {
                (l, l)
            };
            let (r_low, r_high) = if Some(condition_var) == r_var {
                let r_node = right.node(condition_var, r.node_index());
                (r_node.low(), r_node.high())
            } else {
                (r, r)
            };

            let result_low = lookup_table(l_low.as_bool(), r_low.as_bool())
                .map(NodePointer::terminal)
                .or_else(|| tasks.resolve(l_low, r_low));

            let result_high = lookup_table(l_high.as_bool(), r_high.as_bool())
                .map(NodePointer::terminal)
                .or_else(|| tasks.resolve(l_high, r_high));

            if let (Some(result_low), Some(result_high)) = (result_low, result_high) {
                if result_low == result_high {  // No decision here.
                    tasks.save(l, r, result_low);
                } else {                        // Create decision node if it does not exist.
                    let node = Node(result_low, result_high);
                    if let Some(existing) = nodes.find(condition_var, node) {
                        tasks.save(l, r, existing);
                    } else {
                        let new_pointer = output.push_node(condition_var, node);
                        nodes.insert(condition_var, node, new_pointer);
                        tasks.save(l, r, new_pointer);
                    }
                }

                // Task done.
                task_stack.pop();
            } else {
                if result_low.is_none() {
                    task_stack.push((l_low, r_low));
                }
                if result_high.is_none() {
                    task_stack.push((l_high, r_high));
                }
            }

        }
    }

    //println!("Node stats: {:?}", nodes.stats);
    //println!("Task stats: {:?}", tasks.stats);

    let result = tasks.resolve(left.root(), right.root()).unwrap_or_else(|| {
        panic!("When the main loop is finished, this task must be completed.")
    });

    if let Some(constant) = result.as_bool() {
        Bdd::mk_const(constant)
    } else {
        output.set_root(result);
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::bdd;
    use crate::bdd_u16::{VariableId, Bdd};

    fn v1() -> VariableId {
        return VariableId(0);
    }
    fn v2() -> VariableId {
        return VariableId(1);
    }
    fn v3() -> VariableId {
        return VariableId(2);
    }
    fn v4() -> VariableId {
        return VariableId(3);
    }

    fn mk_small_test_bdd() -> Bdd {
        Bdd::mk_var(v3(), true).and(&Bdd::mk_var(v4(), true).not())
    }

    #[test]
    fn bdd_not_preserves_equivalence() {
        let a = Bdd::mk_var(v1(), true);
        let not_a = Bdd::mk_var(v1(), false);
        let b = Bdd::mk_var(v2(), true);
        let not_b = Bdd::mk_var(v2(), false);
        assert_eq!(a.not(), not_a);
        assert_eq!(bdd!(!(a & not_b)), bdd!(not_a | b));
    }

    #[test]
    fn bdd_mk_not() {
        let bdd = mk_small_test_bdd();
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();
        assert_eq!(bdd, bdd!(!(!bdd)));
        assert_eq!(tt, bdd!(!ff));
        assert_eq!(ff, bdd!(!tt));
    }

    #[test]
    fn bdd_mk_and() {
        let bdd = mk_small_test_bdd(); // v3 & !v4
        let v3 = Bdd::mk_var(v3(), true);
        let v4 = Bdd::mk_var(v4(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();
        assert_eq!(bdd, bdd!(v3 & (!v4)));
        assert_eq!(bdd, bdd!(tt & bdd));
        assert_eq!(bdd, bdd!(bdd & tt));
        assert_eq!(ff, bdd!(ff & bdd));
        assert_eq!(ff, bdd!(bdd & ff));
        assert_eq!(bdd, bdd!(bdd & bdd));
    }

    #[test]
    fn bdd_mk_or() {
        let bdd = mk_small_test_bdd(); // v3 & !v4
        let v3 = Bdd::mk_var(v3(), true);
        let v4 = Bdd::mk_var(v4(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();
        assert_eq!(bdd, bdd!(!((!v3) | v4))); // !(!v3 | v4) <=> v3 & !v4
        assert_eq!(tt, bdd!(tt | bdd));
        assert_eq!(tt, bdd!(bdd | tt));
        assert_eq!(bdd, bdd!(ff | bdd));
        assert_eq!(bdd, bdd!(bdd | ff));
        assert_eq!(bdd, bdd!(bdd | bdd));
    }

    #[test]
    fn bdd_mk_xor() {
        let bdd = mk_small_test_bdd(); // v3 & !v4
        let v3 = Bdd::mk_var(v3(), true);
        let v4 = Bdd::mk_var(v4(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();

        assert_eq!(bdd!(!bdd), bdd!(tt ^ bdd));
        assert_eq!(bdd!(!bdd), bdd!(bdd ^ tt));
        assert_eq!(ff, bdd!(bdd ^ bdd));
        assert_eq!(bdd, bdd!(ff ^ bdd));
        assert_eq!(bdd, bdd!(bdd ^ ff));
        assert_eq!(bdd, bdd!(v3 & (v3 ^ v4)));
    }

    #[test]
    fn bdd_mk_imp() {
        let bdd = mk_small_test_bdd(); // v3 & !v4
        let v3 = Bdd::mk_var(v3(), true);
        let v4 = Bdd::mk_var(v4(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();

        assert_eq!(tt, bdd!(ff => bdd));
        assert_eq!(bdd!(!bdd), bdd!(bdd => ff));
        assert_eq!(bdd, bdd!(tt => bdd));
        assert_eq!(tt, bdd!(bdd => tt));
        assert_eq!(tt, bdd!(bdd => bdd));
        assert_eq!(bdd, bdd!(!(v3 => v4))); // !(v3 => v4) <=> v3 & !v4
    }

    #[test]
    fn bdd_mk_and_not() {
        let bdd = mk_small_test_bdd();
        let not_bdd = bdd.not();
        let v3 = Bdd::mk_var(v3(), true);
        let v4 = Bdd::mk_var(v4(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();

        assert_eq!(bdd, v3.and_not(&v4));
        assert_eq!(not_bdd, tt.and_not(&bdd));
        assert_eq!(ff, bdd.and_not(&tt));
        assert_eq!(ff, ff.and_not(&bdd));
        assert_eq!(bdd, bdd.and_not(&ff));
    }

    #[test]
    fn bdd_mk_iff() {
        let bdd = mk_small_test_bdd(); // v3 & !v4
        let v3 = Bdd::mk_var(v3(), true);
        let v4 = Bdd::mk_var(v4(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();

        assert_eq!(bdd, bdd!(bdd <=> tt));
        assert_eq!(bdd, bdd!(tt <=> bdd));
        assert_eq!(bdd!(!bdd), bdd!(ff <=> bdd));
        assert_eq!(bdd!(!bdd), bdd!(bdd <=> ff));
        assert_eq!(tt, bdd!(bdd <=> bdd));
        assert_eq!(bdd, bdd!(v3 & (!(v4 <=> v3))));
    }

    #[test]
    fn bdd_constants() {
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();
        assert!(tt.is_true());
        assert!(ff.is_false());
        assert_eq!(ff, bdd!((tt & ff)));
        assert_eq!(tt, bdd!((tt | ff)));
        assert_eq!(tt, bdd!((tt ^ ff)));
        assert_eq!(ff, bdd!((tt => ff)));
        assert_eq!(ff, bdd!((tt <=> ff)));
    }

    #[test]
    fn simple_identities_syntactic() {
        let a = Bdd::mk_var(v1(), true);
        let tt = Bdd::mk_true();
        let ff = Bdd::mk_false();

        assert_eq!(ff, bdd!((ff & a)));
        assert_eq!(a, bdd!((ff | a)));
        assert_eq!(tt, bdd!((ff => a)));
        assert_eq!(bdd!(!a), bdd!((a => ff)));
        assert_eq!(tt, bdd!((a => a)));
    }

    #[test]
    fn bdd_de_morgan() {
        // !(a * b * !c) <=> (!a + !b + c)
        let v1 = Bdd::mk_var(v1(), true);
        let v2 = Bdd::mk_var(v2(), true);
        let v3 = Bdd::mk_var(v3(), true);

        let left = bdd!(!(v1 & (v2 & (!v3))));
        let right = bdd!(((!v1) | (!v2)) | v3);

        assert_eq!(left, right);
        assert!(bdd!(left <=> right).is_true());
    }

    #[test]
    fn nontrivial_identity_syntactic() {
        // dnf (!a * !b * !c) + (!a * !b * c) + (!a * b * c) + (a * !b * c) + (a * b * !c)
        //                                    <=>
        // cnf            !(!a * b * !c) * !(a * !b * !c) * !(a * b * c)
        let a = Bdd::mk_var(v1(), true);
        let b = Bdd::mk_var(v2(), true);
        let c = Bdd::mk_var(v3(), true);

        let d1 = bdd!(((!a) & (!b)) & (!c));
        let d2 = bdd!(((!a) & (!b)) & c);
        let d3 = bdd!(((!a) & b) & c);
        let d4 = bdd!((a & (!b)) & c);
        let d5 = bdd!((a & b) & (!c));

        let c1 = bdd!((a | (!b)) | c);
        let c2 = bdd!(((!a) | b) | c);
        let c3 = bdd!(((!a) | (!b)) | (!c));

        let cnf = bdd!(((c1 & c2) & c3));
        let dnf = bdd!(((((d1 | d2) | d3) | d4) | d5));

        assert_eq!(cnf, dnf);
        assert!(bdd!((cnf <=> dnf)).is_true());
        //assert_eq!(20.0, cnf.cardinality());
    }
/*
    #[test]
    fn invert_input() {
        let (var1, var2, var3, var4) = (v1(), v2(), v3(), v4());
        let v1 = Bdd::mk_var(v1(), true);
        let v2 = Bdd::mk_var(v2(), true);
        let v3 = Bdd::mk_var(v3(), true);

        let original: Bdd = bdd!(!(v1 & (v2 & (!v3))));
        let invert_v1: Bdd = bdd!(!((!v1) & (v2 & (!v3))));
        let invert_v2: Bdd = bdd!(!(v1 & ((!v2) & (!v3))));
        let invert_v3: Bdd = bdd!(!(v1 & (v2 & v3)));

        assert!(Bdd::fused_binary_flip_op(
            (&invert_v1, None),
            (&original, Some(var1)),
            None,
            crate::op_function::iff
        )
            .is_true());
        assert!(Bdd::fused_binary_flip_op(
            (&original, Some(var2)),
            (&invert_v2, None),
            None,
            crate::op_function::iff
        )
            .is_true());
        assert!(Bdd::fused_binary_flip_op(
            (&invert_v3, None),
            (&original, Some(var3)),
            None,
            crate::op_function::iff
        )
            .is_true());
        assert!(Bdd::fused_binary_flip_op(
            (&original, Some(var4)),
            (&original, None),
            None,
            crate::op_function::iff
        )
            .is_true());
    }
*/

}