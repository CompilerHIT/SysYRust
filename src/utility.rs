use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
    ptr::eq,
    {pin::Pin, ptr::NonNull},
};

/// 进行一个简单的log,而不是printfln!<br>
/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!<br>
/// 史诗级更新<br>
/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        // crate::log_file!("log", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_file {
    ($file:expr, $($arg:tt)*) => {{
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open($file)
        .expect("Failed to open log file");

        writeln!(file, $($arg)*).expect("Failed to write to log file");
    }};
}

// 该宏用来进行不换行的文件log
#[macro_export]
macro_rules! log_file_uln {
    ($file:expr, $($arg:tt)*) => {{

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open($file)
        .expect("Failed to open log file");
        write!(file, $($arg)*).expect("Failed to write to log file");
    }};
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
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

impl<T> Deref for ObjPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for ObjPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
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

impl<T> Hash for ObjPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.as_ref(), state);
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
