/*
 * @Author: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @Date: 2024-08-13 11:06:14
 * @LastEditors: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @LastEditTime: 2024-08-15 17:48:22
 * @FilePath: \thread\src\main.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::Duration;

/**
 * @description: 线程和join等待线程结束
 * @return {*}
 */
fn thread_and_join() {
    let test_vec = vec![1, 2, 3, 4, 5];
    let (tx, rx) = mpsc::channel();
    let thread_handle = thread::spawn(move || {
        for i in 1..10 {
            println!("Number {} from spawned thread", i);
            thread::sleep(Duration::from_millis(1000));
        }
        println!("exit spawned thread, test_vec: {:?}", test_vec);
        tx.send("thread over").unwrap();
    });
    for i in 1..5 {
        println!("Number {} from main thread", i);
        thread::sleep(Duration::from_millis(100));
    }
    println!("wait for spawned thread to finish");
    let received = rx.recv().unwrap(); // block until receive message
    println!("received message: {}", received);
    thread_handle.join().unwrap();
    println!("exit process!");
}


fn thread_and_mutex() {
    struct TeseCounter {
        counter: u32,
    }
    let test_counter = {
        TeseCounter {
            counter: 0,
        }
    };
    let test_counter_no_arc = {
        TeseCounter {
            counter: 0,
        }
    };

    // 创建一个 Arc 实例，内部持有一个 test_counter
    let counter = Arc::new(Mutex::new(test_counter));

    let mut handles = vec![];

    for _ in 0..10 {
        // 克隆一个 Arc 实例，这只是增加引用计数，而不会复制底层的数据。这样你可以在不同的线程中共享同一数据。
        let counter: Arc<Mutex<TeseCounter>> = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut test_counter = counter.lock().unwrap();
            test_counter.counter += 1;
            println!("counter: {}, ", test_counter.counter);
        });
        handles.push(handle);
    }

    // for _ in 0..10 {
    //     let handle = thread::spawn( || {
    //         let mut test_counter = &muttest_counter_no_arc;
    //         test_counter.counter += 1;
    //         println!("counter: {}, ", test_counter.counter);
    //     });

    //     handles.push(handle);
    // }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("result counter: {}", counter.lock().unwrap().counter);
}

fn main() {
    // 测试线程
    // thread_and_join();

    // 测试mutex
    thread_and_mutex();
    let x = 1;
    let mut y = x;
    y += 1;
    println!("x: {}, y: {}", x, y);
}
