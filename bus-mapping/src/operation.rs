//! Collection of structs and functions used to:
//! - Define the internals of a [`MemoryOp`], [`StackOp`] and [`StorageOp`].
//! - Define the actual operation types and a wrapper over them (the
//!   [`Operation`] enum).
//! - Define structures that interact with operations such as
//!   [`OperationContainer`].
pub(crate) mod container;

pub use super::evm::{GlobalCounter, MemoryAddress, StackAddress};
use crate::eth_types::{Address, Word};
pub use container::OperationContainer;
use core::cmp::Ordering;
use core::fmt;
use core::fmt::Debug;

/// Marker that defines whether an Operation performs a `READ` or a `WRITE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RW {
    /// Marks op as READ.
    READ,
    /// Marks op as WRITE.
    WRITE,
}

impl RW {
    /// Returns true if the RW corresponds internally to a [`READ`](RW::READ).
    pub const fn is_read(&self) -> bool {
        match self {
            RW::READ => true,
            RW::WRITE => false,
        }
    }

    /// Returns true if the RW corresponds internally to a [`WRITE`](RW::WRITE).
    pub const fn is_write(&self) -> bool {
        !self.is_read()
    }
}

/// Enum used to differenciate between EVM Stack, Memory and Storage operations.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Target {
    /// Means the target of the operation is the Memory.
    Memory,
    /// Means the target of the operation is the Stack.
    Stack,
    /// Means the target of the operation is the Storage.
    Storage,
    /// Means the target of the operation is the TxAccessListAccount.
    TxAccessListAccount,
    /// Means the target of the operation is the TxAccessListStorageSlot.
    TxAccessListStorageSlot,
    /// Means the target of the operation is the TxRefund.
    TxRefund,
    /// Means the target of the operation is the Account.
    Account,
    /// Means the target of the operation is the AccountDestructed.
    AccountDestructed,
}

/// Trait used for Operation Kinds.
pub trait Op: Eq + Ord {
    /// Turn the Generic Op into an OpEnum so that we can match it into a
    /// particular Operation Kind.
    fn into_enum(self) -> OpEnum;
}

/// Represents a [`READ`](RW::READ)/[`WRITE`](RW::WRITE) into the memory implied
/// by an specific [`OpcodeId`](crate::evm::opcodes::ids::OpcodeId) of the
/// [`ExecStep`](crate::circuit_input_builder::ExecStep).
#[derive(Clone, PartialEq, Eq)]
pub struct MemoryOp {
    /// RW
    pub rw: RW,
    /// Memory Address
    pub address: MemoryAddress,
    /// Value
    pub value: u8,
}

impl fmt::Debug for MemoryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MemoryOp { ")?;
        f.write_fmt(format_args!(
            "{:?}, addr: {:?}, value: 0x{:02x}",
            self.rw, self.address, self.value
        ))?;
        f.write_str(" }")
    }
}

impl MemoryOp {
    /// Create a new instance of a `MemoryOp` from it's components.
    pub fn new(rw: RW, address: MemoryAddress, value: u8) -> MemoryOp {
        MemoryOp { rw, address, value }
    }

    /// Returns the internal [`RW`] which says whether the operation corresponds
    /// to a Read or a Write into memory.
    pub const fn rw(&self) -> RW {
        self.rw
    }

    /// Returns the [`Target`] (operation type) of this operation.
    pub const fn target(&self) -> Target {
        Target::Memory
    }

    /// Returns the [`MemoryAddress`] associated to this Operation.
    pub const fn address(&self) -> &MemoryAddress {
        &self.address
    }

    /// Returns the bytes read or written by this operation.
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Op for MemoryOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::Memory(self)
    }
}

impl PartialOrd for MemoryOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemoryOp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

/// Represents a [`READ`](RW::READ)/[`WRITE`](RW::WRITE) into the stack implied
/// by an specific [`OpcodeId`](crate::evm::opcodes::ids::OpcodeId) of the
/// [`ExecStep`](crate::circuit_input_builder::ExecStep).
#[derive(Clone, PartialEq, Eq)]
pub struct StackOp {
    /// RW
    pub rw: RW,
    /// Stack Address
    pub address: StackAddress,
    /// Value
    pub value: Word,
}

impl fmt::Debug for StackOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StackOp { ")?;
        f.write_fmt(format_args!(
            "{:?}, addr: {:?}, val: {:?}",
            self.rw, self.address, self.value
        ))?;
        f.write_str(" }")
    }
}

impl StackOp {
    /// Create a new instance of a `MemoryOp` from it's components.
    pub const fn new(rw: RW, address: StackAddress, value: Word) -> StackOp {
        StackOp { rw, address, value }
    }

    /// Returns the internal [`RW`] which says whether the operation corresponds
    /// to a Read or a Write into memory.
    pub const fn rw(&self) -> RW {
        self.rw
    }

    /// Returns the [`Target`] (operation type) of this operation.
    pub const fn target(&self) -> Target {
        Target::Stack
    }

    /// Returns the [`StackAddress`] associated to this Operation.
    pub const fn address(&self) -> &StackAddress {
        &self.address
    }

    /// Returns the [`Word`] read or written by this operation.
    pub const fn value(&self) -> &Word {
        &self.value
    }
}

impl Op for StackOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::Stack(self)
    }
}

impl PartialOrd for StackOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StackOp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

/// Represents a [`READ`](RW::READ)/[`WRITE`](RW::WRITE) into the storage
/// implied by an specific [`OpcodeId`](crate::evm::opcodes::ids::OpcodeId) of
/// the [`ExecStep`](crate::circuit_input_builder::ExecStep).
#[derive(Clone, PartialEq, Eq)]
pub struct StorageOp {
    /// RW
    pub rw: RW,
    /// Account Address
    pub address: Address,
    /// Storage Key
    pub key: Word,
    /// Storage Value after the operation
    pub value: Word,
    /// Storage Value before the operation
    pub value_prev: Word,
}

impl fmt::Debug for StorageOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StorageOp { ")?;
        f.write_fmt(format_args!(
            "{:?}, addr: {:?}, key: {:?}, val_prev: {:?}, val: {:?}",
            self.rw, self.address, self.key, self.value_prev, self.value,
        ))?;
        f.write_str(" }")
    }
}

impl StorageOp {
    /// Create a new instance of a `StorageOp` from it's components.
    pub const fn new(
        rw: RW,
        address: Address,
        key: Word,
        value: Word,
        value_prev: Word,
    ) -> StorageOp {
        StorageOp {
            rw,
            address,
            key,
            value,
            value_prev,
        }
    }

    /// Returns the internal [`RW`] which says whether the operation corresponds
    /// to a Read or a Write into storage.
    pub const fn rw(&self) -> RW {
        self.rw
    }

    /// Returns the [`Target`] (operation type) of this operation.
    pub const fn target(&self) -> Target {
        Target::Storage
    }

    /// Returns the [`Address`] corresponding to this storage operation.
    pub const fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the [`Word`] used as key for this operation.
    pub const fn key(&self) -> &Word {
        &self.key
    }

    /// Returns the [`Word`] read or written by this operation.
    pub const fn value(&self) -> &Word {
        &self.value
    }

    /// Returns the [`Word`] at key found previous to this operation.
    pub const fn value_prev(&self) -> &Word {
        &self.value_prev
    }
}

impl Op for StorageOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::Storage(self)
    }
}

impl PartialOrd for StorageOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StorageOp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.address.cmp(&other.address) {
            Ordering::Equal => self.key.cmp(&other.key),
            ord => ord,
        }
    }
}

/// Represents a change in the Account AccessList implied by a `BeginTx`,
/// `EXTCODECOPY`, `EXTCODESIZE`, `BALANCE`, `SELFDESTRUCT`, `*CALL`* or
/// `CREATE*` step.
#[derive(Clone, PartialEq, Eq)]
pub struct TxAccessListAccountOp {
    /// Transaction ID: Transaction index in the block starting at 1.
    pub tx_id: usize,
    /// Account Address
    pub address: Address,
    /// Value after the operation
    pub value: bool,
    /// Value before the operation
    pub value_prev: bool,
}

impl fmt::Debug for TxAccessListAccountOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TxAccessListAccountOp { ")?;
        f.write_fmt(format_args!(
            "tx_id: {:?}, addr: {:?}, val_prev: {:?}, val: {:?}",
            self.tx_id, self.address, self.value_prev, self.value
        ))?;
        f.write_str(" }")
    }
}

impl PartialOrd for TxAccessListAccountOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxAccessListAccountOp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tx_id.cmp(&other.tx_id) {
            Ordering::Equal => self.address.cmp(&other.address),
            ord => ord,
        }
    }
}

impl Op for TxAccessListAccountOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::TxAccessListAccount(self)
    }
}

/// Represents a change in the Storage AccessList implied by an `SSTORE` or
/// `SLOAD` step of the [`ExecStep`](crate::circuit_input_builder::ExecStep).
#[derive(Clone, PartialEq, Eq)]
pub struct TxAccessListStorageSlotOp {
    /// Transaction ID: Transaction index in the block starting at 1.
    pub tx_id: usize,
    /// Account Address
    pub address: Address,
    /// Storage Key
    pub key: Word,
    /// Value after the operation
    pub value: bool,
    /// Value before the operation
    pub value_prev: bool,
}

impl fmt::Debug for TxAccessListStorageSlotOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TxAccessListStorageSlotOp { ")?;
        f.write_fmt(format_args!(
            "tx_id: {:?}, addr: {:?}, key: {:?}, val_prev: {:?}, val: {:?}",
            self.tx_id, self.address, self.key, self.value_prev, self.value
        ))?;
        f.write_str(" }")
    }
}

impl PartialOrd for TxAccessListStorageSlotOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxAccessListStorageSlotOp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tx_id.cmp(&other.tx_id) {
            Ordering::Equal => match self.address.cmp(&other.address) {
                Ordering::Equal => self.key.cmp(&other.key),
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl Op for TxAccessListStorageSlotOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::TxAccessListStorageSlot(self)
    }
}

/// Represents a change in the Transaction Refund AccessList implied by an
/// `SSTORE`, `STOP`, `RETURN` or `REVERT` step of the
/// [`ExecStep`](crate::circuit_input_builder::ExecStep).
#[derive(Clone, PartialEq, Eq)]
pub struct TxRefundOp {
    /// RW
    pub rw: RW,
    /// Transaction ID: Transaction index in the block starting at 1.
    pub tx_id: usize,
    /// Refund Value after the operation
    pub value: Word,
    /// Refund Value before the operation
    pub value_prev: Word,
}

impl fmt::Debug for TxRefundOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TxRefundOp { ")?;
        f.write_fmt(format_args!(
            "{:?} tx_id: {:?}, val_prev: {:?}, val: {:?}",
            self.rw, self.tx_id, self.value_prev, self.value
        ))?;
        f.write_str(" }")
    }
}

impl PartialOrd for TxRefundOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxRefundOp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tx_id.cmp(&other.tx_id)
    }
}

impl Op for TxRefundOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::TxRefund(self)
    }
}

/// Represents a field parameter of the Account that can be accessed via EVM
/// execution.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccountField {
    /// Account Nonce
    Nonce,
    /// Account Balance
    Balance,
    /// Account Code Hash
    CodeHash,
}

/// Represents a change in the Account field implied by a `BeginTx`,
/// `EXTCODECOPY`, `EXTCODESIZE`, `BALANCE`, `SELFDESTRUCT`, `*CALL`*,
/// `CREATE*`, `STOP`, `RETURN` or `REVERT` step.
#[derive(Clone, PartialEq, Eq)]
pub struct AccountOp {
    /// RW
    pub rw: RW,
    /// Account Address
    pub address: Address,
    /// Field
    pub field: AccountField,
    /// Field Value after the operation
    pub value: Word,
    /// Field Value before the operation
    pub value_prev: Word,
}

impl fmt::Debug for AccountOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccountOp { ")?;
        f.write_fmt(format_args!(
            "{:?}, addr: {:?}, field: {:?}, val_prev: {:?}, val: {:?}",
            self.rw, self.address, self.field, self.value_prev, self.value
        ))?;
        f.write_str(" }")
    }
}

impl PartialOrd for AccountOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccountOp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.address.cmp(&other.address) {
            Ordering::Equal => self.field.cmp(&other.field),
            ord => ord,
        }
    }
}

impl Op for AccountOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::Account(self)
    }
}

/// Represents an Account destruction implied by a `SELFDESTRUCT` step of the
/// [`ExecStep`](crate::circuit_input_builder::ExecStep).
#[derive(Clone, PartialEq, Eq)]
pub struct AccountDestructedOp {
    /// Transaction ID: Transaction index in the block starting at 1.
    pub tx_id: usize,
    /// Account Address
    pub address: Address,
    /// Value after the operation
    pub value: bool,
    /// Value before the operation
    pub value_prev: bool,
}

impl fmt::Debug for AccountDestructedOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccountDestructedOp { ")?;
        f.write_fmt(format_args!(
            "tx_id: {:?}, addr: {:?}, val_prev: {:?}, val: {:?}",
            self.tx_id, self.address, self.value_prev, self.value
        ))?;
        f.write_str(" }")
    }
}

impl PartialOrd for AccountDestructedOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccountDestructedOp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tx_id.cmp(&other.tx_id) {
            Ordering::Equal => self.address.cmp(&other.address),
            ord => ord,
        }
    }
}

impl Op for AccountDestructedOp {
    fn into_enum(self) -> OpEnum {
        OpEnum::AccountDestructed(self)
    }
}

// TODO: https://github.com/appliedzkp/zkevm-circuits/issues/225
// pub struct CallContextOp{}

/// Generic enum that wraps over all the operation types possible.
/// In particular [`StackOp`], [`MemoryOp`] and [`StorageOp`].
#[derive(Debug, Clone)]
pub enum OpEnum {
    /// Stack
    Stack(StackOp),
    /// Memory
    Memory(MemoryOp),
    /// Storage
    Storage(StorageOp),
    /// TxAccessListAccount
    TxAccessListAccount(TxAccessListAccountOp),
    /// TxAccessListStorageSlot
    TxAccessListStorageSlot(TxAccessListStorageSlotOp),
    /// TxRefund
    TxRefund(TxRefundOp),
    /// Account
    Account(AccountOp),
    /// AccountDestructed
    AccountDestructed(AccountDestructedOp),
    /* TODO: https://github.com/appliedzkp/zkevm-circuits/issues/225
     * CallContext(CallContextOp), */
}

/// Operation is a Wrapper over a type that implements Op with a GlobalCounter.
#[derive(Debug, Clone)]
pub struct Operation<T: Op> {
    gc: GlobalCounter,
    /// True when this Operation is a revert of its mirror
    revert: bool,
    op: T,
}

impl<T: Op> PartialEq for Operation<T> {
    fn eq(&self, other: &Self) -> bool {
        self.op.eq(&other.op) && self.gc == other.gc
    }
}

impl<T: Op> Eq for Operation<T> {}

impl<T: Op> PartialOrd for Operation<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Op> Ord for Operation<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.op.cmp(&other.op) {
            Ordering::Equal => self.gc.cmp(&other.gc),
            ord => ord,
        }
    }
}

impl<T: Op> Operation<T> {
    /// Create a new Operation from an `op` with a `gc`
    pub fn new(gc: GlobalCounter, op: T) -> Self {
        Self {
            gc,
            revert: false,
            op,
        }
    }

    /// Return this `Operation` `gc`
    pub fn gc(&self) -> GlobalCounter {
        self.gc
    }

    /// Return this `Operation` `op`
    pub fn op(&self) -> &T {
        &self.op
    }

    // /// Matches over an `Operation` returning the [`Target`] of the iternal
    // op /// it stores inside.
    // pub const fn target(&self) -> Target {
    //     self.op.target()
    // }
    //     match self {
    //         Operation::Memory(_) => Target::Memory,
    //         Operation::Stack(_) => Target::Stack,
    //         Operation::Storage(_) => Target::Storage,
    //     }
    // }

    // /// Returns true if the Operation hold internally is a [`StackOp`].
    // pub const fn is_stack(&self) -> bool {
    //     matches!(*self, Operation::Stack(_))
    // }

    // /// Returns true if the Operation hold internally is a [`MemoryOp`].
    // pub const fn is_memory(&self) -> bool {
    //     matches!(*self, Operation::Memory(_))
    // }

    // /// Returns true if the Operation hold internally is a [`StorageOp`].
    // pub const fn is_storage(&self) -> bool {
    //     matches!(*self, Operation::Storage(_))
    // }

    // /// Transmutes the internal (unlabeled) repr of the operation contained
    // /// inside of the enum into a [`StackOp`].
    // pub fn into_stack_unchecked(self) -> StackOp {
    //     match self {
    //         Operation::Stack(stack_op) => stack_op,
    //         _ => panic!("Broken Invariant"),
    //     }
    // }

    // /// Transmutes the internal (unlabeled) repr of the operation contained
    // /// inside of the enum into a [`MemoryOp`].
    // pub fn into_memory_unchecked(self) -> MemoryOp {
    //     match self {
    //         Operation::Memory(memory_op) => memory_op,
    //         _ => panic!("Broken Invariant"),
    //     }
    // }

    // /// Transmutes the internal (unlabeled) repr of the operation contained
    // /// inside of the enum into a [`StorageOp`].
    // pub fn into_storage_unchecked(self) -> StorageOp {
    //     match self {
    //         Operation::Storage(storage_op) => storage_op,
    //         _ => panic!("Broken Invariant"),
    //     }
    // }
}

#[cfg(test)]
mod operation_tests {
    use super::*;

    #[test]
    fn unchecked_op_transmutations_are_safe() {
        let stack_op =
            StackOp::new(RW::WRITE, StackAddress::from(1024), Word::from(0x40));

        let stack_op_as_operation =
            Operation::new(GlobalCounter(1), stack_op.clone());

        let memory_op = MemoryOp::new(RW::WRITE, MemoryAddress(0x40), 0x40);

        let memory_op_as_operation =
            Operation::new(GlobalCounter(1), memory_op.clone());

        assert_eq!(stack_op, stack_op_as_operation.op);
        assert_eq!(memory_op, memory_op_as_operation.op)
    }
}
