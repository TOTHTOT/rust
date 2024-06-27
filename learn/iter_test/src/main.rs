/*
 * @Description: 迭代器学习
 * @Author: TOTHTOT
 * @Date: 2024-06-26 17:46:32
 * @LastEditTime: 2024-06-27 11:25:05
 * @LastEditors: TOTHTOT
 * @FilePath: \rust\learn\iter_test\src\main.rs
 */
#[derive(Debug)]
 struct Shirt{
    color: String,
    size: u32,
 }

fn main() {
    let list = vec![1, 2, 3];
    let list_iter = list.iter();
    for val in list_iter {
        println!("Got: {}", val);
    }

    let list = vec![1, 2, 3];
    let list_iter:Vec<_> = list.iter().map(|x| x + 1).collect();
    println!("{:?}", list);
    println!("{:?}", list_iter);

    // 创建一个向量包含多个衣服
    let shirt = vec![
        Shirt{
            color: String::from("blue"),
            size: 10,
        },
        Shirt{
            color: String::from("red"),
            size: 10,
        },
        Shirt{
            color: String::from("blue"),
            size: 20,
        },
    ];
    // 通过 filter() 从向量中过滤出颜色为 "blue" 的元素, 并收集到向量中
    let blue_shirts:Vec<_> = shirt.iter().filter(|s|s.color == "blue").collect();
    println!("color: {:?}", blue_shirts);

    // 通过 filter() 从向量中过滤出尺寸为 10 的元素, 并收集到向量中
    let blue_shirts:Vec<_> = shirt.iter().filter(|s|s.size == 10).collect();
    println!("size: {:?}", blue_shirts);
}
