fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
    // 引用第一个数据
    let mut max = &list[0];
    for item in list {
        if item > max {
            max = item;
        }
    }
    max
}

fn main() {
    let num_list = vec![1, 2, 3, 4, 5];
    println!("largest = {}", largest(&num_list));
}
