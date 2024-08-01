/*
 * @Description: 
 * @Author: TOTHTOT
 * @Date: 2024-07-05 13:40:25
 * @LastEditTime: 2024-08-01 17:46:34
 * @LastEditors: TOTHTOT
 * @FilePath: \rust\project\bom_manage\src\main.rs
 */
use std::error::Error;
use bom_manage_lib::bom_manage::BomManageCtrl;
use std::io::{self, Write};
use clap::{Command, Arg};

fn main() -> Result<(), Box<dyn Error>> {
    // 数据文件地址
    const DATA_FILE: &str = "data_resource.db";
    // 表名
    const TABLE_NAME: &str = "bom_data";
    // 初始化数据库
    let mut bom_manage_ctrl = match BomManageCtrl::new(DATA_FILE, TABLE_NAME) {
        Ok(bom_manage_ctrl) => bom_manage_ctrl,
        Err(err) => {
            println!("初始化数据库失败: {}", err);
            Err(err)
        }?,
    };

    // 创建一个新的应用实例
    let matches = Command::new("MyApp")
    .version("1.0")
    .author("Author Name <author@example.com>")
    .about("Does awesome things")
    .arg(Arg::new("config")
        .help("Sets a custom config file")
        .required(false)
        .index(1))
    .get_matches();

    // loop {
        
    // }
    Ok(())
}
