use super::Entity;
use nonmax::NonMaxU32;

/// A sparse data structure of V.
pub struct EntitySparseSet<V> {
    dense: Vec<V>,
    sparse: Vec<Option<NonMaxU32>>,
}

impl<V> Default for EntitySparseSet<V> {
    fn default() -> Self {
        Self {
            dense: vec![],
            sparse: vec![],
        }
    }
}

impl<V> EntitySparseSet<V> {
    /// Returns a reference to the value for `Entity`.
    ///
    /// Returns `None` if `index` does not have a value in the sparse set.
    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&V> {
        let dense_index = self.sparse.get(entity.index() as usize)?;
        if let Some(index) = *dense_index {
            // SAFETY: if the sparse index points to something in the dense vec, it exists
            unsafe { Some(self.dense.get_unchecked(index.get() as usize)) }
        } else {
            None
        }
    }

    /// Inserts the `entity` key and  `value` pair into this sparse
    /// set.
    pub fn insert(&mut self, entity: Entity, value: V) {
        let index = entity.index() as usize;
        if let Some(&sparse) = self.sparse.get(index) {
            if let Some(dense_index) = sparse {
                // SAFETY: if the sparse index points to something in the dense vec, it exists
                unsafe { *self.dense.get_unchecked_mut(dense_index.get() as usize) = value };
            } else {
                // SAFETY: if the sparse index points to something in the dense vec, it exists
                unsafe {
                    *self.sparse.get_unchecked_mut(index) =
                        Some(NonMaxU32::new_unchecked(self.dense.len() as u32))
                }
                self.dense.push(value);
            }
        } else {
            self.sparse.resize(index + 1, None);
            // SAFETY: the sparse index exists after resize.
            unsafe {
                *self.sparse.get_unchecked_mut(index) =
                    Some(NonMaxU32::new_unchecked(self.dense.len() as u32))
            }
            self.dense.push(value);
        }
    }

    /// Returns `true` if the collection contains a value for the specified `index`.
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        let dense_index = self.sparse.get(entity.index() as usize);
        if let Some(index) = dense_index {
            index.is_some()
        } else {
            false
        }
    }

    /// Removes all of the values stored within.
    pub fn clear(&mut self) {
        self.sparse.clear();
        self.dense.clear();
    }

    /// Inserts the `entity` key and  `value` pair into this sparse from slice
    ///
    /// This operation is safe if every `entity` in slice and dense are unique.
    pub fn insert_from_slice_unique(&mut self, entites: &[Entity], values: &[V]) {
        assert!(
            entites.len() == values.len() && self.dense.len() + entites.len() < u32::MAX as usize
        );
        let mut dst_len = self.dense.len();
        let count = entites.len();
        self.dense.reserve(count);
        // SAFETY: dense has reserved enough capacity.
        // The slices cannot overlap because mutable references are exclusive.
        unsafe {
            std::ptr::copy_nonoverlapping(
                values.as_ptr(),
                self.dense.as_mut_ptr().add(dst_len),
                count,
            );
            self.dense.set_len(dst_len + count)
        }
        entites.iter().copied().for_each(|e| {
            let index = e.index() as usize;
            // SAFETY: the sparse index exists after resize.
            if index >= self.sparse.len() {
                self.sparse.resize(index + 1, None)
            }
            unsafe {
                *self.sparse.get_unchecked_mut(index) =
                    Some(NonMaxU32::new_unchecked(dst_len as u32))
            }
            dst_len += 1;
        })
    }
}
