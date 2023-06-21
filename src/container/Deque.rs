// 支持中间修改的双向链表实现
// TODO 用来加速图着色分配

// use std::{collections::{HashMap, VecDeque}, rc::Rc};

// pub struct Deque<T> {
//     fronts:HashMap<i32,i32>,
//     nexts:HashMap<i32,i32>,
//     available_ids: VecDeque<i32>,
//     begin:i32,
//     end:i32,
//     vals:HashMap<i32,T>,
// }

// pub struct Cursor<T> {
//     id:i32,
//     list:Rc<Deque<T>>,
// }

// impl<T>  Deque<T> {

//     pub fn cursor_front_mut(&self)->Option<Cursor<T>> {
//         if self.vals.len()==0 {
//             return None;
//         }
//         return Some(Cursor { id: self.begin, list:Rc::new(*self) });
//     }

// }

// impl<T> Cursor<T> {
    
// }



