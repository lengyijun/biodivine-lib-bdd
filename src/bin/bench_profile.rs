//! You can use this target for profiling your benchmarks. Either call your benchmark function
//! from the main here, or just copy paste it. Don't forget to compile in --release for
//! optimisations.

use biodivine_lib_bdd::{bdd, BddNode};
use biodivine_lib_bdd::bdd_u16::{Bdd, VariableId};

fn main() {
    let num_vars = 26;
    let shift_by = (64 - num_vars) / 2;
    let mut result = Bdd::mk_false();
    for x in 0..(num_vars / 2) {
        let x1 = Bdd::mk_var(VariableId((x + shift_by) as u32), true);
        let x2 = Bdd::mk_var(VariableId(((x + num_vars / 2) + shift_by) as u32), true);
        result = bdd!(result | (x1 & x2));
    }

    println!("Nodes: {}", result.node_count());
    println!("Sizeof: {}", std::mem::size_of::<BddNode>());
}
