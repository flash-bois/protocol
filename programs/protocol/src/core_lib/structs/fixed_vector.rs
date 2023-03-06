use checked_decimal_macro::num_traits::ToPrimitive;
use std::{
    ops::Range,
    slice::{Iter, IterMut},
};

#[derive(Debug, Clone, Copy)]
pub struct FixedSizeVector<T, const N: usize>
where
    T: Default + PartialEq,
{
    head: u8,
    elements: [T; N],
}

impl<T, const N: usize> Default for FixedSizeVector<T, N>
where
    T: Default + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> FixedSizeVector<T, N>
where
    T: Default + PartialEq,
{
    pub fn new() -> Self {
        assert!(N.to_u8().is_some(), "size expands u8 range");

        Self {
            head: 0,
            elements: [(); N].map(|_| T::default()),
        }
    }

    pub fn iter<'a>(&'a self) -> Option<Iter<'a, T>> {
        if self.head == 0 {
            return None;
        }

        let range = ..self.head_usize();

        Some(self.elements.get(range)?.iter())
    }

    pub fn iter_mut<'a>(&'a mut self) -> Option<IterMut<'a, T>> {
        if self.head == 0 {
            return None;
        }

        let range = ..self.head_usize();

        Some(self.elements.get_mut(range)?.iter_mut())
    }

    pub fn find_mut(&mut self, search: &T) -> Option<&mut T> {
        if let Some(mut iter) = self.iter_mut() {
            return iter.find(|el| *search == **el);
        }

        None
    }

    pub fn enumerate_find_mut(&mut self, search: &T) -> Option<(usize, &mut T)> {
        if let Some(iter) = self.iter_mut() {
            return iter.enumerate().find(|(_id, pos)| *search == **pos);
        }

        None
    }

    pub fn delete(&mut self, id: usize) {
        // checks if id is before vector head
        assert!(self.index_before_head(id), "bad index");

        // move element that has to be delete to last position, shifting rest by -1
        // then it removes last position
        if let Some(iter) = self.iter_mut() {
            iter.into_slice().get_mut(id..).unwrap().rotate_left(1);
            self.remove();
        }
    }

    pub fn indexes(&self) -> Range<usize> {
        0..(self.head as usize)
    }

    // parse head u8 to usize
    fn head_usize(&self) -> usize {
        // unwrap because we assert if N can be fitted inside u8 on creation
        self.head.to_usize().unwrap()
    }

    /// checks if index is in useful range
    fn index_before_head(&self, id: usize) -> bool {
        self.head > 0 && id < self.head_usize()
    }

    /// checks if index is in allocated range
    fn index_in_capacity(&self, id: usize) -> bool {
        id < N
    }

    /// returns immutable element under the index, does not check if it is before head,
    /// only check if it in array allocated area
    pub fn get(&self, id: usize) -> Option<&T> {
        if self.index_in_capacity(id) {
            self.elements.get(id)
        } else {
            None
        }
    }

    /// returns mutable element under the index, does not check if it is before head,
    /// only check if it in array allocated area
    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        if self.index_in_capacity(id) {
            self.elements.get_mut(id)
        } else {
            None
        }
    }

    /// returns mutable element under the index,
    /// check if it is in initialized range
    pub fn get_mut_checked(&mut self, id: usize) -> Option<&mut T> {
        if self.index_before_head(id) {
            self.elements.get_mut(id)
        } else {
            None
        }
    }

    /// returns immutable mutable element under the index,
    /// check if it is in initialized range
    pub fn get_checked(&self, id: usize) -> Option<&T> {
        if self.index_before_head(id) {
            self.elements.get(id)
        } else {
            None
        }
    }

    pub fn add(&mut self, el: T) -> Result<(), ()> {
        let head = self.head_usize();

        if !self.index_in_capacity(head) {
            return Err(());
        }

        *self.get_mut(head).ok_or(())? = el;
        self.head += 1;

        Ok(())
    }

    pub fn remove(&mut self) -> Option<&T> {
        if self.head == 0 {
            return None;
        }

        self.head -= 1;
        self.get(self.head_usize())
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.head > 0 {
            self.get_mut(self.head_usize() - 1)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.head > 0 {
            self.get(self.head_usize() - 1)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
    struct TestStruct {
        size: u8,
        quantity: u32,
    }

    #[test]
    fn test_new() {
        let new_vec = FixedSizeVector::<TestStruct, 3>::new();

        assert_eq!(new_vec.head, 0u8);
        assert_eq!(new_vec.last(), None);
    }

    #[test]
    fn test_add() {
        let mut new_vec = FixedSizeVector::<TestStruct, 128>::new();

        let t1 = TestStruct {
            size: 11,
            quantity: 11,
        };

        let t2 = TestStruct {
            size: 22,
            quantity: 22,
        };

        let t3 = TestStruct {
            size: 33,
            quantity: 33,
        };

        assert!(
            new_vec.add(t1.clone()).is_ok()
                && new_vec.add(t2.clone()).is_ok()
                && new_vec.add(t3.clone()).is_ok(),
            "can add to vec"
        );

        assert_eq!(new_vec.head, 3);
        assert_eq!(new_vec.last(), Some(&t3));
        assert_eq!(new_vec.get(128), None);
        assert_eq!(new_vec.get(127), Some(&TestStruct::default()));
        assert_eq!(
            new_vec.get(new_vec.head_usize()),
            Some(&TestStruct::default())
        );

        let mut range = new_vec.indexes();
        assert_eq!(range.next(), Some(0));
        assert_eq!(range.next(), Some(1));
        assert_eq!(range.next(), Some(2));
        assert_eq!(range.next(), None);

        assert_eq!(new_vec.remove(), Some(&t3));
        assert_eq!(new_vec.remove(), Some(&t2));
        assert_eq!(new_vec.remove(), Some(&t1));
        assert_eq!(new_vec.remove(), None);
        //new_vec.remove(),
    }
}
