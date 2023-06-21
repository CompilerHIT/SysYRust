

// 一个简单的bitmap,用来统计spilling情况,用位图
pub struct Bitmap{
    arr :Vec<u64>,
}

impl Bitmap {
    pub fn andOther(&mut self,other:&Bitmap) {
        while other.cap() >self.arr.len() {
            self.arr.push(0);
        }
        let i=0;
        while i<self.cap()&&i<other.cap() {
            self.arr[i]=self.arr[i]&other.arr[i];
        }
    }
    pub fn orOther(&mut self,other :&Bitmap) {
        while other.cap() >self.arr.len() {
            self.arr.push(0);
        }
        let i=0;
        while i<self.cap()&&i<other.cap() {
            self.arr[i]=self.arr[i]|other.arr[i];
        }
    }
    
    pub fn count(&self) -> usize{
        let mut out=0;
        for v in &self.arr {
            let mut tmp =*v;
            for _ in 0..64 {
                if tmp%2==1 {out+=1}
                tmp/=2;
            }
        }
        out
    }


}


impl Bitmap {
    pub fn new()->Bitmap {
        Bitmap{arr:Vec::new()}
    }
    pub fn with_cap(cap:usize)->Bitmap {
        Bitmap { arr: Vec::with_capacity(cap) }
    }
    pub fn insert(&mut self,i:usize){
        while i /64 >=self.arr.len() {
            self.arr.push(0);
        }
        let mut v=&mut self.arr[i/64];
        *v=*v | (i as u64%64)
    }
    
    pub fn cap(&self) -> usize{
        self.arr.len()
    }
    
    pub fn and(a: &Bitmap,b:&Bitmap)->Bitmap {
        let mut i:usize=a.cap();
        if i<b.cap() {
            i=b.cap();
        }
        let mut out=Bitmap::with_cap(i);
        i=0;
        while i<a.cap()&&i<b.cap() {
            out.arr[i]=a.arr[i]&b.arr[i];
        }
        out
    }
    pub fn or(a:&Bitmap,b:&Bitmap)->Bitmap {
        let mut i:usize=a.cap();
        if i<b.cap() {
            i=b.cap();
        }
        let mut out=Bitmap::with_cap(i);
        i=0;
        while i<a.cap()&&i<b.cap() {
            out.arr[i]=a.arr[i]&b.arr[i];
        }
        out
    }


}