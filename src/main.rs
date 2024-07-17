mod spinlock;
use std::thread;
// #[inline(never)]
static Q:spinlock::SpinLock<i64>=spinlock::SpinLock::new(0);
fn add(){
    for i in 0..1000000{
        *Q.lock()+=1
    }
}
fn main() {
    println!("Hello, world!");
    let mut v=vec![];
    for _ in 0..100{
        v.push(thread::spawn(add))
    }
    for i in v{
        i.join();
    }
    println!("{}",*Q.lock())
}
