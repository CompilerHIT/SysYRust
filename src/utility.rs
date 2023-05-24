use std::{
    fmt::Debug,
    hash::Hash,
    ptr::eq,
    {pin::Pin, ptr::NonNull},
};

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum ScalarType {
    Void,
    Int,
    Float,
}

/// 一个封装的指针
/// 通过调用as_ref和as_mut来获得可变或不可变引用
pub struct ObjPtr<T>(NonNull<T>);

impl<T> ObjPtr<T> {
    pub fn as_ref<'a>(self) -> &'a T {
        unsafe { self.0.as_ref() }
    }

    pub fn as_mut<'a>(mut self) -> &'a mut T {
        unsafe { self.0.as_mut() }
    }

    pub fn new(ptr: &T) -> Self {
        unsafe { Self(NonNull::new_unchecked(ptr as *const _ as *mut _)) }
    }
}

impl<T> Clone for ObjPtr<T> {
    fn clone(&self) -> Self {
        ObjPtr(self.0.clone())
    }
}

impl<T> PartialEq for ObjPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        eq(self.as_ref(), other.as_ref())
    }
}

impl<T> Eq for ObjPtr<T> {}

impl<T> Copy for ObjPtr<T> {}

impl<T: Debug + 'static> Debug for ObjPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// 一个内存池
/// 通过调用put申请一块内存，然后函数返回ObjPtr获得其指针
pub struct ObjPool<T> {
    data: Vec<Pin<Box<T>>>,
}

impl<T> ObjPool<T> {
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn put(&mut self, value: T) -> ObjPtr<T> {
        self.data.push(Box::pin(value));
        let p = self.data.last_mut().unwrap();
        ObjPtr::new(p)
    }

    pub fn free_all(&mut self) {
        self.data.clear()
    }
}
