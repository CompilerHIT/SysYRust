// 支持中间修改的双向链表实现
// TODO 用来加速图着色分配

use std::{collections::{HashMap, VecDeque, LinkedList}, rc::Rc, ptr::NonNull};

pub struct Deque<T> {
    fronts:HashMap<i32,i32>,
    nexts:HashMap<i32,i32>,
    available_ids: VecDeque<i32>,
    begin:i32,
    end:i32,
    len:usize,
    vals:HashMap<i32,T>,
}

pub struct CursorMut<'a,T> {
    id:i32,
    list:&'a Deque<T>,
}

impl<T>  Deque<T> {

    pub fn new()->Deque<T> {
        Deque { fronts: HashMap::new(), nexts: HashMap::new(), available_ids: VecDeque::from([0,1]), begin: 0, end: 0, len: 0, vals: HashMap::new() }
    }

    pub fn cursor_end_mut(&mut self)->Option<CursorMut<T>> {
        if self.vals.len()==0 {
            return None;
        }
        return Some(CursorMut {id:-1,list:self});
    }

    pub fn cursor_front_mut(&mut self)->Option<CursorMut<T>> {
        if self.vals.len()==0 {
            return None;
        }
        return Some(CursorMut {id:-1,list:self});
    }

    pub fn len(&self)->usize {
        return  self.len;
    }

    // pub fn push_front(&mut self,val:T) {
    //     // 取出一个待用id,
    //     if self.available_ids.len()<=1 {
    //         let end=self.available_ids.back().unwrap();
    //         self.available_ids.push_back(end+1);
    //     }
    //     let id=self.available_ids.pop_front().unwrap();
    //     if self.len==0 {
    //         self.begin=id;
    //         self.end=id;
    //         self.vals.insert(id, val);


    //     }
    // }
    
    // pub fn push_back(&mut self,val:T) {

    // }
    // pub fn pop_front(&mut self)->Option<T>{

    // }
    // pub fn pop_back(&mut self)->Option<T>{

    // }

}

impl<'a,T> CursorMut<'a,T> {
    
}



