use crate::bdd_u16::{NodePointer, VariableId};
use std::ops::{Shl, Shr};
use std::convert::TryFrom;

// 2 bits per block = 4 variables per block
const VAR_BLOCK_SIZE: u32 = 4;

impl NodePointer {
    /// Constant representation of the zero pointer.
    pub fn zero() -> NodePointer {
        NodePointer(0b_0000_0000_0000_0000)
    }

    /// Constant representation of the one pointer.
    pub fn one() -> NodePointer {
        NodePointer(0b_1000_0000_0000_0000)
    }

    /// Make a new terminal pointer (0/1) with the specified value.
    pub fn terminal(value: bool) -> NodePointer {
        match value {
            true => Self::one(),
            false => Self::zero(),
        }
    }

    /// If this node is a terminal, return the terminal value.
    pub fn as_bool(&self) -> Option<bool> {
        if self.is_terminal() {
            Some(self.0 != 0)
        } else {
            None
        }
    }

    /// True if this pointer is the one terminal.
    pub fn is_one(&self) -> bool {
        self.0 == 0b_1000_0000_0000_0000
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn flip_if_terminal(&self) -> NodePointer {
        if self.is_one() {
            NodePointer::zero()
        } else if self.is_zero() {
            NodePointer::one()
        } else {
            *self
        }
    }

    /// Create a new pointer using the given `variable` and `node_index`.
    ///
    /// This method panics if the `node_index` is not addressable for the given `variable`.
    ///
    /// (However, we assume the `VariableId` is safely addressable in this space as it
    /// should have been checked when it was created!)
    pub fn new(variable: VariableId, node_index: usize) -> NodePointer {
        // 0b00..0b11
        let id_in_block = variable.0 % VAR_BLOCK_SIZE;
        // 0b0000..0b1111; Id of the 4-variable block.
        // Low blocks are starting with 0, high blocks are starting with 1.
        let block_id = variable.0 / VAR_BLOCK_SIZE;
        let is_low_block = block_id & 0b1000 == 0;
        // 0b000..0b111; The number of trailing zeroes in the representation.
        let block_rank = if is_low_block {
            0b0111 - block_id & 0b0111  // In low blocks, first blocks have the most leading zeroes.
        } else {
            block_id & 0b0111           // whereas in high blocks, it is the last blocks.
        };

        // This conversion should generally succeed, because address space overflow should occur
        // much sooner. However, it can happen when deserializing corrupted data.
        let pointer: u16 = u16::try_from(node_index).unwrap_or_else(|_| {
            panic!("Value {} is too large for a 16-bit Bdd pointer.", node_index);
        });

        // Check if it is safe to shift the node_index by the necessary amount of bits:
        let total_shift = block_rank + 4;   // +1 for mark, +1 for low/high, +2 for id
        if pointer.shl(total_shift).shr(total_shift) != pointer {
            panic!("Pointer ({},{}) cannot be allocated with 16-bit addresses.", variable.0, node_index);
        }

        // Now we can just use "unsafe" shift operations to pack all data into a single u16
        // First, add variable id inside the block and the high/low block flag.
        let high_flag: u16 = if is_low_block { 0 } else { 1 };
        let pointer: u16 = (pointer.shl(3) + (id_in_block.shl(1) as u16)) + high_flag;
        // Then add the block marker and shift to match the block rank.
        let pointer: u16 = (pointer.shl(1u16) + 1u16).shl(block_rank);

        NodePointer(pointer)
    }

    /// Returns true if the node is `one` or `zero`.
    pub fn is_terminal(&self) -> bool {
        self.0 & 0b0111_1111_1111_1111 == 0
    }

    /// Tests if the pointer is one of the 124 reserved values outside of the normal range.
    ///
    /// If the value is not a pointer, behaviour of other methods on such value is undefined.
    pub fn is_pointer(&self) -> bool {
        self.is_terminal() || self.0.trailing_zeros() <= 8
    }

    /// A non trivial pointer is a valid pointer that is not terminal. From these pointers,
    /// we can extract the pointer variable and node index.
    pub fn is_non_trivial(&self) -> bool {
        self.is_pointer() && !self.is_terminal()
    }

    /// Returns the `VariableId` of this node.
    ///
    /// The behaviour of this methods is undefined for terminal nodes.
    pub fn variable_id(&self) -> VariableId {
        debug_assert!(self.is_non_trivial());
        // 0b000..0b111
        let block_rank = self.0.trailing_zeros();
        // +1 for the block marker bit
        let sans_rank = self.0.shr(block_rank + 1);
        let is_low_block = sans_rank & 0b1 == 0;
        // 0b00..0b11
        let id_in_block = sans_rank.shr(1) & 0b11u16;
        // 0b0000..0b1111; Id of the 4-variable block
        let block_id = if is_low_block {
            // Low blocks are 0b0000..0b0111, with lowest having the biggest rank.
            0b111 - block_rank
        } else {
            // High blocks are 0b1000..0b1111, with highest having the biggest rank.
            0b1000 + block_rank
        };
        let id = VAR_BLOCK_SIZE * block_id + u32::from(id_in_block);
        VariableId(id)
    }

    /// Returns the node index defined by this pointer.
    ///
    /// The behaviour of this method is undefined for terminal nodes.
    pub fn node_index(&self) -> usize {
        debug_assert!(self.is_non_trivial());
        let block_rank = self.0.trailing_zeros();
        // +1 for block marker, +1 for high/low bit, +2 for block bit width.
        usize::from(self.0.shr(block_rank + 4))
    }
}

impl From<u16> for NodePointer {
    fn from(value: u16) -> Self {
        NodePointer(value)
    }
}

impl From<NodePointer> for u16 {
    fn from(pointer: NodePointer) -> Self {
        pointer.0
    }
}

#[cfg(test)]
mod tests {
    use crate::bdd_u16::{NodePointer, VariableId};

    #[test]
    fn node_pointer_encoding() {
        // Last variable in the lowest block of variables:
        let pointer = NodePointer::new(VariableId(0b0000_0011), 0b10101);
        assert_eq!(0b10101_110_1000_0000, pointer.0);
        assert_eq!(VariableId(0b0000_0011), pointer.variable_id());
        assert_eq!(0b10101, pointer.node_index());
        // Highest block of variables:
        let pointer = NodePointer::new(VariableId(0b0011_1101), 0b01010);
        assert_eq!(0b01010_011_1000_0000, pointer.0);
        assert_eq!(VariableId(0b0011_1101), pointer.variable_id());
        assert_eq!(0b01010, pointer.node_index());
        // Middle block of variables:
        let pointer = NodePointer::new(VariableId(0b0010_0000), 0b101010_101010);
        assert_eq!(0b101010_101010_001_1, pointer.0);
        assert_eq!(VariableId(0b0010_0000), pointer.variable_id());
        assert_eq!(0b101010_101010, pointer.node_index());
    }

    #[test]
    fn node_pointer_basics() {
        let one = NodePointer::one();
        let zero = NodePointer::zero();
        let non_trivial = NodePointer::new(VariableId(10), 0b110011);

        let invalid = NodePointer(0b1100_0000_0000_0000);
        assert!(one.is_pointer());
        assert!(zero.is_pointer());
        assert!(non_trivial.is_pointer());
        assert!(!invalid.is_pointer());

        assert!(one.is_terminal());
        assert!(zero.is_terminal());
        assert!(!non_trivial.is_terminal());

        assert!(!one.is_non_trivial());
        assert!(!zero.is_non_trivial());
        assert!(non_trivial.is_non_trivial());
    }

    #[test]
    #[should_panic]
    fn node_pointer_index_overflow() {
        NodePointer::new(VariableId(0), usize::MAX - 100);
    }

    #[test]
    #[should_panic]
    fn node_pointer_address_overflow() {
        NodePointer::new(VariableId(0), 0b100000);
    }

}