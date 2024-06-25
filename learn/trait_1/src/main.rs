/*
 * @Description: trait 项目
 * @Author: TOTHTOT
 * @Date: 2024-06-20 09:55:42
 * @LastEditTime: 2024-06-20 16:19:03
 * @LastEditors: TOTHTOT
 * @FilePath: \rust\learn\trait_1\src\main.rs
 */
// 也可以 trait_lib_1::*; 引用所有模块
use trait_lib_1::{NewsArchive, Summary, Tweet};

// trait 作为参数
fn noticy(item:&impl Summary) {
    println!("Breaking news! {}", item.summarize());
}
// 限制只有含有 Copy trait 的类型变量才允许交换
fn swap<T:Copy>(a:&mut T, b:&mut T)
{
    let temp = *a;
    *a = *b;
    *b = temp;
}

// fn test_refrece(){
//     let x = 1;
//     let a = x;

//     println!("a = {}, x = {}", a, x);
//     let b = &a;
//     println!("a = {}, b = {}, x = {}", a, b, x);
// }

fn main() {
    let na = NewsArchive {
        headline: String::from("Penguins win the Stanley Cup Championship!"),
        location: String::from("Pittsburgh, PA, USA"),
        author: String::from("Iceburgh"),
        content: String::from("The Pittsburgh Penguins once again are the best."),
    };
    let tweet = Tweet {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        retweet: false,
    };
    
    println!("tweet = {}", tweet.summarize());
    println!("tweet info = {:?}", tweet);
    println!("na = {}, {}", na.summarize(), na.get_headline());

    noticy(&tweet);

    let mut a:f64 = 1.1;
    let mut b:f64 = 2.2;
    println!("before a = {}, b = {}", a, b);
    swap(&mut a, &mut b);
    println!("after a = {}, b = {}", a, b);
    // 此时会报错, 因为NewsArchive Tweet 没有实现 Copy trait
    // swap(&mut na, &mut tweet);

    // test_refrece();
}
