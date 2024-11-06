/*
 * @Author: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @Date: 2024-08-23 10:25:06
 * @LastEditors: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @LastEditTime: 2024-08-23 15:41:34
 * @FilePath: \pointer\src\main.rs
 * @Description:rust学习之智能指针
 */
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

// 这是元组结构体
struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

// 引入解引用特性
impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 引入离开作用域释放特性
impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        println!("drop");
    }
}

fn test_drop() {
    {
        let x = 5;
        let y = MyBox::new(5);
    
        println!("x = {}, y = {}", x, *y);
    }
    println!("Hello, world!");
}

/**
 * @description: 测试线程给数字加1
 * @param {u32} mode == 1 随机编号+1; == 2 按编号顺序+1
 * @return {*}
 */
fn test_thread(mode: u32) {
    let mut handles = vec![];
    let counter_au = Arc::new(AtomicUsize::new(0));
    let counter_aum = Arc::new((Mutex::new(0), Condvar::new()));

    for i in 0..10 {
        let thread_counter_au = Arc::clone(&counter_au);
        let thread_counter_aum = Arc::clone(&counter_aum);

        let handle = thread::spawn(move || {
            match mode {
                1 => { // 随机编号+1
                    let prev = thread_counter_au.fetch_add(1, Ordering::Relaxed);
                    println!("Hello from the thread {}, thread_counter = {}", i, prev + 1);
                },
                2 => { // 按编号顺序+1
                    let (lock, cvar) = &*thread_counter_aum;
                    let mut num = lock.lock().unwrap();

                    // 等待条件变量符合
                    while *num != i {
                        println!("cur = {}, wait = {}", i, *num);
                        num = cvar.wait(num).unwrap();
                    }
                    *num += 1;
                    println!("Hello from the thread {}, thread_counter = {}", i, *num);

                    // 通知其他线程条件变量已经改变
                    cvar.notify_all()
                },
                _ => {
                    println!("not support mode = {}", mode)
                }
            }

        });
        handles.push(handle);
    }

    // 回收线程
    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    test_drop();

    test_thread(2);
}
