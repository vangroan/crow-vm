use std::rc::Rc;

#[derive(Clone)]
pub struct Str(Rc<String>);

impl Str {
    pub unsafe fn as_ptr(self) -> *const String {
        Rc::into_raw(self.0)
    }

    pub unsafe fn from_ptr(string_rc: *const String) -> Self {
        Self(Rc::from_raw(string_rc))
    }
}
