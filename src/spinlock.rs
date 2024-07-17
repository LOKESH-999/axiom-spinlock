use core::sync::atomic::{AtomicBool,Ordering::{Acquire,Release}};
use core::hint::spin_loop;
use core::cell::UnsafeCell;
use core::ops::{Deref,DerefMut};

pub struct SpinLock<T>{
    data:UnsafeCell<T>,
    locked:AtomicBool
}
pub struct SpinGuard<'a,T>{
    guard:&'a SpinLock<T>
}
impl<'a,T> Drop for SpinGuard<'a,T>{
    fn drop(&mut self){
        self.guard.locked.store(false, Release)
    }
}
impl<T> SpinLock<T>{
    #[allow(unused)]
    pub const fn new(data:T)->Self{
        SpinLock { 
            data: UnsafeCell::new(data),
            locked: AtomicBool::new(false) 
        }
    }
    #[allow(unused)]
    pub fn lock(&self)->SpinGuard<T>{
        while self.locked.swap(true,Acquire) { //Acquire ,why not AcqRel because swap ensours that all the writes are visible to its consecutive read operation
            spin_loop()
        }

        SpinGuard { 
            guard:&self
        }
    }
}
impl<T> Deref for SpinGuard<'_,T>{
    type Target=T;
    fn deref(&self)->&T{
        unsafe {
            & *(self.guard.data.get() )   
        }
    }
}

impl<T> DerefMut for SpinGuard<'_,T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{
            &mut *self.guard.data.get()
        }
    }
}