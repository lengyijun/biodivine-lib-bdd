use crate::{bdd, Bdd, BddVariableSet};
use test::Bencher;
use crate::_impl_bdd::bench_task_generator::{spawn_tasks, spawn_tasks_2};


fn ripple_carry_adder(b: &mut Bencher, num_vars: u16) {
    let vars = BddVariableSet::new_anonymous(num_vars);
    let variables = vars.variables();
    b.iter(|| {
        let mut result = vars.mk_false();
        for x in 0..(num_vars / 2) {
            let x1 = vars.mk_var(variables[x as usize]);
            let x2 = vars.mk_var(variables[(x + num_vars / 2) as usize]);
            result = bdd!(result | (x1 & x2));
        }
        result
    });
}
/*
#[bench]
fn ripple_carry_adder_4(bencher: &mut Bencher) {
    ripple_carry_adder(bencher, 4);
}

#[bench]
fn ripple_carry_adder_8(bencher: &mut Bencher) {
    ripple_carry_adder(bencher, 8);
}

#[bench]
fn ripple_carry_adder_16(bencher: &mut Bencher) {
    ripple_carry_adder(bencher, 16);
}
 */


#[bench]
fn minus_1000(bencher: &mut Bencher) {
    let a = Bdd::from_string(&std::fs::read_to_string("inputs/minus_1000_a.bdd").unwrap());
    let b = Bdd::from_string(&std::fs::read_to_string("inputs/minus_1000_b.bdd").unwrap());
    println!("A:{}, B:{}", a.size(), b.size());
    a.and_not(&b);
    println!("Spawned: {:?}", spawn_tasks_2(&a, &b));
    bencher.iter(|| {
        spawn_tasks_2(&a, &b)
        //a.and_not(&b)
    });
}

#[bench]
fn minus_10000(bencher: &mut Bencher) {
    let a = Bdd::from_string(&std::fs::read_to_string("inputs/minus_10000_a.bdd").unwrap());
    let b = Bdd::from_string(&std::fs::read_to_string("inputs/minus_10000_b.bdd").unwrap());
    println!("A:{}, B:{}", a.size(), b.size());
    a.and_not(&b);
    println!("Spawned: {:?}", spawn_tasks_2(&a, &b));
    bencher.iter(|| {
        spawn_tasks_2(&a, &b)
        //a.and_not(&b)
    });
}