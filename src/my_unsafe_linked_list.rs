use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

struct UnsafeLinkedList<'a, T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    _marker: PhantomData<&'a Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    val: Option<T>,
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
}

impl<'a, T> UnsafeLinkedList<'a, T> {
    fn create() -> Self {
        unsafe {
            let head = Box::into_non_null(Box::new(Node {
                val: None,
                prev: None,
                next: None,
            }));
            let tail = Box::into_non_null(Box::new(Node {
                val: None,
                prev: Some(head),
                next: None,
            }));

            (*head.as_ptr()).next = Some(tail);

            UnsafeLinkedList {
                head: Some(head),
                tail: Some(tail),
                _marker: PhantomData,
            }
        }
    }

    fn push(&self, val: T) {
        unsafe {
            let next = (*self.head.unwrap().as_ptr()).next;
            let curr = Box::into_non_null(Box::new(Node {
                val: Some(val),

                prev: self.head,
                next: (*self.head.unwrap().as_ptr()).next,
            }));

            (*self.head.unwrap().as_ptr()).next = Some(curr);
            (*next.unwrap().as_ptr()).prev = Some(curr);
        }
    }
}

unsafe fn free_list<T>(start: Option<NonNull<Node<T>>>) {
    unsafe {
        let mut curr = start;
        while let Some(data) = curr {
            let next_node = (*data.as_ptr()).next;
            let _ = Box::from_raw(data.as_ptr());
            curr = next_node;
        }
    }
}

impl<T> Drop for UnsafeLinkedList<'_, T> {
    fn drop(&mut self) {
        unsafe {
            free_list(self.head);
        }
    }
}

struct MyIter<'a, T> {
    curr: Option<NonNull<Node<T>>>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> Iterator for MyIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let curr_ptr = self.curr?;
            let next_node_opt = (*curr_ptr.as_ptr()).next;
            if let Some(next_node) = next_node_opt {
                self.curr = Some(next_node);
                return (*next_node.as_ptr()).val.as_ref();
            }
            None
        }
    }
}

struct MyOwningIter<T> {
    curr: Option<NonNull<Node<T>>>,
    head: Option<NonNull<Node<T>>>,
}

impl<T> IntoIterator for UnsafeLinkedList<'_, T> {
    type Item = T;
    type IntoIter = MyOwningIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let list = ManuallyDrop::new(self);
        MyOwningIter {
            curr: list.head,
            head: list.head,
        }
    }
}

impl<'a, T> IntoIterator for &UnsafeLinkedList<'a, T> {
    type Item = &'a T;
    type IntoIter = MyIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        MyIter {
            curr: self.head,
            _marker: PhantomData,
        }
    }
}

impl<T> Iterator for MyOwningIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let curr_ptr = self.curr?;
            let next_node_opt = (*curr_ptr.as_ptr()).next;
            if let Some(next_node) = next_node_opt {
                self.curr = Some(next_node);
                return (*next_node.as_ptr()).val.take();
            }
            None
        }
    }
}

impl<T> Drop for MyOwningIter<T> {
    fn drop(&mut self) {
        unsafe {
            free_list(self.head);
        }
    }
}
