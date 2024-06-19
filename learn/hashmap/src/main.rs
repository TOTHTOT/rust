// 引入哈希表
use std::collections::HashMap;


fn main() {
    // 创建哈希表
    let mut scores = HashMap::new();
    let team_name = String::from("Blue");
    let team_score = 10;

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    scores.insert(team_name, team_score);

    // 所有权已被移交, 此时不在有效
    // println!("team_name{}, scores:{}", team_name, team_score);
    println!("before HashMap{:?}", scores);

    // 更新哈希表
    scores.insert(String::from("Blue"), 25);
    println!("after HashMap{:?}", scores);

    // 计算单词出现数量
    let str = String::from("hello world helloworld hello world");
    let mut word_num = HashMap::new();
    for word in str.split_whitespace() {
        let count = word_num.entry(word).or_insert(0);
        *count += 1;
    }
    println!("{:?}", word_num);
}
