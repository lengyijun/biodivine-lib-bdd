use crate::bdd_u16::VariableId;
use std::convert::TryFrom;

impl VariableId {}

impl From<VariableId> for usize {
    fn from(value: VariableId) -> Self {
        // We assume we will be running only on platforms where this is ok, and it seems
        // that the compiler can optimize it away completely in such cases.
        usize::try_from(value.0).unwrap()
    }
}
