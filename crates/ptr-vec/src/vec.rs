use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    mem::transmute,
};

use derive_more::Display;
use thiserror::Error;

#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq)]
pub enum VecError {
    #[display(fmt = "capacity overflow")]
    CapacityOverflow,
    #[display(fmt = "allocation error")]
    AllocError(Layout),
}

/// An untyped vector that holds any data with a specified [`Layout`]
pub struct PtrVec {
    layout: Layout,
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl Drop for PtrVec {
    fn drop(&mut self) {
        if self.cap != 0 && self.data_size() != 0 {
            unsafe { dealloc(self.ptr, self.array_layout()) }
        }
    }
}

impl PtrVec {
    /// Creates a new [`PtrVec`] without allocating
    pub fn new(layout: Layout) -> Self {
        Self {
            ptr: unsafe { transmute(layout.align()) },
            cap: match layout.size() {
                0 => usize::MAX,
                _ => 0,
            },
            layout: layout.pad_to_align(),
            len: 0,
        }
    }

    /// Gets the current length of the vec
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vec is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the layout of the data stored in this vec
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns the size of the data stored in the vec
    pub fn data_size(&self) -> usize {
        self.layout.size()
    }

    /// Returns the alignment of the data stored in the vec
    pub fn data_align(&self) -> usize {
        self.layout.align()
    }

    /// Returns the layout of the entire data array
    pub fn array_layout(&self) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.array_size(), self.data_align()) }
    }

    /// Returns the size of the entire data array
    pub fn array_size(&self) -> usize {
        self.cap * self.data_size()
    }

    /// Returns the current capacity of the vec
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the first pointer in the vec
    pub fn first(&self) -> Option<*mut u8> {
        if self.len == 0 {
            return None;
        }

        Some(self.ptr)
    }

    /// Returns the last pointer in the vec
    pub fn last(&self) -> Option<*mut u8> {
        self.get(self.len.saturating_sub(1))
    }

    /// Gets the pointer located at `index` in the vec
    pub fn get(&self, index: usize) -> Option<*mut u8> {
        if index >= self.len {
            return None;
        }

        let offset = index * self.layout.size();
        Some(unsafe { self.ptr.add(offset) })
    }

    /// Reduces the length of the vec by `amount`
    pub fn trim(&mut self, amount: usize) {
        self.len = self.len.saturating_sub(amount)
    }

    /// Returns an iterator over every pointer in the vec
    pub fn iter(&self) -> Iter {
        self.iter_from(0)
    }

    /// Returns an iterator over all pointers in the vec starting at the `start` index
    pub fn iter_from(&self, start: usize) -> Iter {
        Iter {
            vec: self,
            index: start,
        }
    }

    /// Attempts to create another data pointer at th end of the vec
    pub fn try_push(&mut self) -> Result<*mut u8, VecError> {
        // check if the layout is a ZST
        if self.layout.size() == 0 {
            self.len += 1;
            return Ok(self.ptr);
        }

        // ensure the capacity is big enough
        let index = self.len;
        let new_len = index.checked_add(1).ok_or(VecError::CapacityOverflow)?;
        self.ensure_cap(new_len)?;
        self.len = new_len;

        // get the pointer to the data
        let offset = index * self.layout.size();
        Ok(unsafe { self.ptr.add(offset) })
    }

    fn ensure_cap(&mut self, cap: usize) -> Result<(), VecError> {
        // check if cap is correct size
        if cap <= self.cap {
            return Ok(());
        }

        // get the new capacity and create a new layout
        let new_cap = cap.max(self.cap * 2);
        let max_size = isize::MAX as usize - (self.layout.align() - 1);
        if new_cap > max_size {
            return Err(VecError::CapacityOverflow);
        }

        // create the new_ptr and check for null
        let new_ptr = match self.cap {
            0 => unsafe {
                alloc(Layout::from_size_align_unchecked(
                    new_cap * self.data_size(),
                    self.data_align(),
                ))
            },
            _ => unsafe { realloc(self.ptr, self.array_layout(), new_cap) },
        };

        // ensure the new ptr is valid
        if new_ptr.is_null() {
            return Err(VecError::AllocError(unsafe {
                Layout::from_size_align_unchecked(new_cap * self.data_size(), self.data_align())
            }));
        }

        // assign the ptr
        self.ptr = new_ptr;
        self.cap = new_cap;
        Ok(())
    }
}

/// An iterator over all the pointers in a [`PtrVec`]
pub struct Iter<'a> {
    vec: &'a PtrVec,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut u8;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.vec.get(self.index)?;
        self.index += 1;
        Some(ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_zst_vec() {
        let layout = Layout::new::<()>();
        let vec = PtrVec::new(layout);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), usize::MAX);
        assert!(vec.is_empty());
    }

    #[test]
    fn empty_sized_vec() {
        let layout = Layout::new::<u32>();
        let vec = PtrVec::new(layout);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn push_zst() {
        let layout = Layout::new::<()>();
        let mut vec = PtrVec::new(layout);
        let ptr = vec.try_push().unwrap();
        assert_eq!(ptr as usize, layout.align());
        assert_eq!(vec.get(0).unwrap() as usize, layout.align());
    }

    #[test]
    fn push_get_sized() {
        let layout = Layout::new::<u32>();
        let mut vec = PtrVec::new(layout);
        let ptr = vec.try_push().unwrap();
        unsafe { (ptr as *mut u32).write(42) }
        let data = unsafe { *(vec.get(0).unwrap() as *mut u32) };
        assert_eq!(data, 42);
    }
}
