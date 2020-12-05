use alloc::collections::BinaryHeap;
use core::{mem::MaybeUninit, slice};

use crate::{archetype::TypeInfo, Archetype, Component};

/// A collection of component types
#[derive(Debug, Clone, Default)]
pub struct ColumnBatchType {
    types: BinaryHeap<TypeInfo>,
}

impl ColumnBatchType {
    /// Create an empty type
    pub fn new() -> Self {
        Self::default()
    }

    /// Update to include `T` components
    pub fn add<T: Component>(&mut self) -> &mut Self {
        self.types.push(TypeInfo::of::<T>());
        self
    }

    /// Construct a [`ColumnBatch`] for entities with these components
    pub fn into_batch(self) -> ColumnBatch {
        ColumnBatch {
            archetype: Archetype::new(self.types.into_sorted_vec()),
        }
    }
}

/// Data describing a collection of entities which have the same set of components
///
/// The "column" name reflects the column-major memory layout exposed via `storage_for`, which
/// matches the internal memory layout of `World` and can hence be used for extremely fast spawning.
pub struct ColumnBatch {
    pub(crate) archetype: Archetype,
}

unsafe impl Send for ColumnBatch {}
unsafe impl Sync for ColumnBatch {}

impl ColumnBatch {
    /// Create a batch with certain component types
    pub fn new(ty: ColumnBatchType) -> Self {
        ty.into_batch()
    }

    /// Allocate storage for `n` additional entities
    pub fn reserve(&mut self, n: u32) {
        self.archetype.reserve(n);
    }

    /// Get storage for `T`s, or `None` if `T` wasn't in the [`ColumnBatchType`]
    pub fn storage_for<T: Component>(&mut self) -> Option<&mut [MaybeUninit<T>]> {
        let base = self.archetype.get::<T>()?;
        Some(unsafe {
            slice::from_raw_parts_mut(base.as_ptr().cast(), self.archetype.capacity() as usize)
        })
    }

    /// Indicate that the first `n` entities have been fully written
    ///
    /// # Safety
    ///
    /// For every `T` in this batch's [`ColumnBatchType`], the first `n` elements of `storage_for<T>()` must
    /// have been written.
    pub unsafe fn set_len(&mut self, n: u32) {
        self.archetype.set_len(n);
    }
}
