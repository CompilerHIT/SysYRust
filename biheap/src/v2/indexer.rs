use std::{rc::Rc, cell::{Ref, RefCell}};

use super::{Weak, RawBiVec, RawNode, BiHeap};

pub struct HeapIndexer <T: Ord> {
    pub(crate) owner: Weak<RawBiVec<T>>, 
    pub(crate) indexer: Weak<RawNode<T>>, 
}

/// Constructors 
impl <T: Ord> BiHeap<T> {
    pub fn max_index(&self) -> Option<HeapIndexer<T>> { 
        #[cfg(not(feature = "threadsafe"))] 
        {
            let bi_vec = self.bi_vec.borrow(); 
            let max = bi_vec.max.get(0)?; 
            Some(HeapIndexer {
                owner : Rc::downgrade(&self.bi_vec),
                indexer: Rc::downgrade(max), 
            }) 
        }
    }
    pub fn min_index(&self) -> Option<HeapIndexer<T>> {
        #[cfg(not(feature = "threadsafe"))] 
        {
            let bi_vec = self.bi_vec.borrow(); 
            let min = bi_vec.min.get(0)?; 
            Some(HeapIndexer {
                owner : Rc::downgrade(&self.bi_vec),
                indexer: Rc::downgrade(min), 
            }) 
        } 
    }
}

pub enum HeapIndexerErrorOwnered <T: Ord> {
    OwnerHeapMismatch, 
    IndexerNodeHasDropped, 
    IndexerNodeOccupied(HeapIndexer<T>), 
} 

pub enum HeapIndexerErrorNotOwned { 
    OwnerHeapMismatch, 
    IndexerNodeHasDropped, 
    IndexerNodeOccupied, 
} 

pub struct ValueViewOnHeap <'a, T: Ord> where Self : 'a {
    pub(crate) _actual: Rc<RefCell<RawNode<T>>>, 
    pub(crate) _data: Option<Ref<'a, RawNode<T>>>, 
} 

/// Consumers  
impl <T: Ord> BiHeap<T> {
    pub fn peek(&self, indexer: &HeapIndexer<T>) -> Result<&T, HeapIndexerErrorNotOwned> {
        type Error = HeapIndexerErrorNotOwned; 
        let me = Rc::downgrade(&self.bi_vec); 
        if !Weak::ptr_eq(&me, &indexer.owner) {
            return Err(Error::OwnerHeapMismatch);   
        }
        let node = indexer.indexer.upgrade().ok_or(Error::IndexerNodeHasDropped)?; 
        let _ret = unsafe { node.try_borrow_unguarded() }; 
        // ret.map_err(|_| Error::IndexerNodeOccupied).map(|x| &x.data) 
        unimplemented!()
    }
}