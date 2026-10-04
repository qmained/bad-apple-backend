use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct Node<T> {
    pub val: Option<T>,
    prev: Option<Weak<RefCell<Node<T>>>>,
    next: Option<Rc<RefCell<Node<T>>>>,
}

#[derive(Clone)]
pub struct MyLinkedList<T> {
    head: Rc<RefCell<Node<T>>>,
    tail: Rc<RefCell<Node<T>>>,
}

impl<T> MyLinkedList<T> {
    pub fn create() -> Self {
        let head = Rc::new(RefCell::new(Node {
            val: None,
            prev: None,
            next: None,
        }));

        let tail = Rc::new(RefCell::new(Node {
            val: None,
            prev: Some(Rc::downgrade(&head)),
            next: None,
        }));

        head.borrow_mut().next = Some(tail.clone());
        MyLinkedList { head, tail }
    }

    pub fn push(&self, val: T) {
        let head = self.head.clone();
        let curr = Rc::new(RefCell::new(Node {
            val: Some(val),
            prev: Some(Rc::downgrade(&head)),
            next: head.borrow().next.clone(),
        }));

        let next = head.borrow().next.clone();
        next.unwrap().clone().borrow_mut().prev = Some(Rc::downgrade(&curr));
        head.borrow_mut().next = Some(curr)
    }

    pub fn iter(&self) -> MyIter<T> {
        self.into_iter()
    }
}

pub struct MyIter<T> {
    curr: Option<Rc<RefCell<Node<T>>>>,
}

impl<T> Iterator for MyIter<T> {
    type Item = Rc<RefCell<Node<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.curr.clone()?.borrow().next.clone();

        if next.clone()?.borrow().val.is_some() {
            self.curr = next.clone();
            return self.curr.clone();
        }

        None
    }
}

impl<T> IntoIterator for MyLinkedList<T> {
    type Item = Rc<RefCell<Node<T>>>;
    type IntoIter = MyIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        MyIter {
            curr: Some(self.head),
        }
    }
}

impl<T> IntoIterator for &MyLinkedList<T> {
    type Item = Rc<RefCell<Node<T>>>;
    type IntoIter = MyIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        MyIter {
            curr: Some(self.head.clone()),
        }
    }
}