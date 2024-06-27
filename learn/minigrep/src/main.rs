use minigrep_lib::*;
use std::env;
use std::process;

fn main() {
    // 传入数据并判断是否有效, 成功就返回congfig对象, 失败就退出程序, 这里没使用 match 匹配
    let config_result = minigrep::Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    // println!("{:#?}", config_result);
    let _file_contens = config_result.run().unwrap_or_else(|err| {
        eprintln!("Config::run failed: {}", err);
        process::exit(1);
    });
    // println!("file contents:\n{}", file_contens);
    // println!("start search...");
    // let result = config_result.search("he", &file_contens);
    // println!("result:\n{:#?}", result);
}

