use crate::{bdd, BddVariableSet};
use test::Bencher;
use crate::bdd_u16::{Bdd, VariableId};

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

fn ripple_carry_adder_u16(b: &mut Bencher, num_vars: u16) {
    b.iter(|| {
        let shift_by = (64 - num_vars) / 2;
        let mut result = Bdd::mk_false();
        for x in 0..(num_vars / 2) {
            let x1 = Bdd::mk_var(VariableId((x + shift_by) as u32), true);
            let x2 = Bdd::mk_var(VariableId(((x + num_vars / 2) + shift_by) as u32), true);
            result = bdd!(result | (x1 & x2));
        }
        result
    });
}

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

/*
#[bench]
#[cfg(feature = "large_benchmarks")]
fn ripple_carry_adder_32(bencher: &mut Bencher) {
    ripple_carry_adder(bencher, 32);
}
*/

#[bench]
fn ripple_carry_adder_4_u16(bencher: &mut Bencher) {
    ripple_carry_adder_u16(bencher, 4);
}

#[bench]
fn ripple_carry_adder_8_u16(bencher: &mut Bencher) {
    ripple_carry_adder_u16(bencher, 8);
}

#[bench]
fn ripple_carry_adder_16_u16(bencher: &mut Bencher) {
    ripple_carry_adder_u16(bencher, 16);
}

#[bench]
fn ripple_carry_adder_32_u16(bencher: &mut Bencher) {
    ripple_carry_adder_u16(bencher, 24);
}

#[bench]
fn ripple_carry_adder_32(bencher: &mut Bencher) {
    ripple_carry_adder(bencher, 24);
}