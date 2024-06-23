//! Reference counted handle.
//!
//! This is a temporary solution until we have an unsafe garbage collector.
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::{Rc, Weak as RcWeak};

// Re-exports
pub use std::cell::{Ref, RefMut};

/// A shared, mutable handle.
pub struct Handle<T>(Rc<RefCell<T>>);

pub struct Weak<T>(RcWeak<RefCell<T>>);

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn into_inner(self) -> T {
        let Self(rc) = self;
        match Rc::try_unwrap(rc) {
            Err(_) => panic!("handle is not unique"),
            Ok(ref_cell) => ref_cell.into_inner(),
        }
    }

    #[inline(always)]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    #[inline(always)]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }

    pub fn ptr_eq(&self, other: &Handle<T>) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn downgrade(&self) -> Weak<T> {
        Weak(Rc::downgrade(&self.0))
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle(self.0.clone())
    }
}

impl<T: fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Handle").field(&*self.0.borrow()).finish()
    }
}

/// A [`Handle`] shared in a circular reference.
pub enum Shared<T> {
    Strong(Handle<T>),
    Weak(Weak<T>),
}

impl<T> Shared<T> {
    pub fn strong(&self) -> Option<&Handle<T>> {
        match self {
            Shared::Strong(handle) => Some(handle),
            Shared::Weak(_) => None,
        }
    }

    pub fn upgrade(&self) -> Option<Handle<T>> {
        match self {
            Shared::Strong(handle) => Some(handle.clone()),
            Shared::Weak(weak) => weak.0.upgrade().map(|rc| Handle(rc)),
        }
    }

    pub fn weak(&self) -> Option<&Weak<T>> {
        match self {
            Shared::Strong(_) => None,
            Shared::Weak(weak) => Some(weak),
        }
    }

    pub fn downgrade(&self) -> Weak<T> {
        match self {
            Shared::Strong(handle) => handle.downgrade(),
            Shared::Weak(weak) => Weak(weak.0.clone()),
        }
    }
}
