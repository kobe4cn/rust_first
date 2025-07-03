use std::{
    collections::LinkedList,
    ops::{Deref, DerefMut, Index},
};
#[derive(Debug)]
struct List<T>(LinkedList<T>);

impl<T> Deref for List<T> {
    type Target = LinkedList<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Index<isize> for List<T> {
    type Output = T;

    fn index(&self, index: isize) -> &Self::Output {
        let size = self.0.len();
        if size == 0 {
            panic!("list is empty");
        }
        let index = index % size as isize;
        let mut ix = index;
        if index < 0 {
            ix = size as isize + index;
        }
        self.0.iter().nth(ix as usize).unwrap()
    }
}
fn main() {
    let mut list: List<u32> = List::default();
    for i in 0..16 {
        list.push_back(i);
    }
    println!("list[0] is: {}", list[0]);
    println!("list[5] is: {}", list[5]);
    println!("list[15] is: {}", list[15]);
    println!("list[16] is: {}", list[16]);
    println!("list[-1] is: {}", list[-1]);
    println!("list[128] is: {}", list[128]);
    println!("list[-128] is: {}", list[-128]);

    assert_eq!(list[0], 0);
    assert_eq!(list[5], 5);
    assert_eq!(list[15], 15);
    assert_eq!(list[16], 0);
    assert_eq!(list[-1], 15);
    assert_eq!(list[128], 0);
    assert_eq!(list[-128], 0);
}

#[test]
fn it_works() {}
