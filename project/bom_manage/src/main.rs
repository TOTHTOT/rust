/*
 * @Author: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @Date: 2024-07-05 13:40:25
 * @LastEditors: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @LastEditTime: 2024-08-13 10:15:22
 * @FilePath: \bom_manage\src\main.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
use bom_manage_lib::bom_manage::*;
use clap::{Arg, ArgMatches, Command};
use rustyline::{error::ReadlineError, history::DefaultHistory, Config, Editor};
use std::error::Error;
use std::io::{self};
use std::process;

// 数据库文件地址
macro_rules! DATA_FILE {
    () => {
        "data_resource.db"
    };
}
// 表名
macro_rules! TABLE_NAME {
    () => {
        "bom_data"
    };
}

// 定义宏来表示命令字符串
macro_rules! COMMAND_ADD {
    () => {
        "add"
    };
}

macro_rules! COMMAND_REMOVE {
    () => {
        "remove"
    };
}

macro_rules! SUBCOMMAND_ALL {
    () => {
        "all"
    };
}

macro_rules! COMMAND_VIEW {
    () => {
        "view"
    };
}

macro_rules! COMMAND_MODIFY {
    () => {
        "modify"
    };
}

/**
 * @description: 获取命令行输入, 并返回参数列表
 * @param {*} progam_name
 * @return {Result<Vec<String>, io::Error>}
 */
fn get_cmd(progam_name: &str, cmd_data: String) -> Result<Vec<String>, io::Error> {
    let input = cmd_data.trim(); // 去除输入两端的空白字符

    if input.len() == 0 {
        // 如果输入为空, 返回错误
        return Err(io::Error::new(io::ErrorKind::Other, "Empty input"));
    }

    // 分割输入字符串为参数列表
    let mut args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    args.insert(0, progam_name.to_string()); // 插入程序名保证获取命令位置准确
    Ok(args)
}

/* fn electronic_component_is_valid(name: &str) -> bool {
    // 检查电子元件名称是否有效, 格式必须是R10K, C20uF, L10uH, 这样的
    !name.is_empty()
} */
/**
 * @description: 添加一个新的电子元件
 * @param {*} matches
 * @return {*}
 */
fn add_electronic_component(
    matches: &ArgMatches,
    bom_manage_ctrl: &mut BomManageCtrl,
) -> Result<(), Box<dyn Error>> {
    let invalid_name = "null".to_string();
    let name = matches.get_one::<String>("name").unwrap_or(&invalid_name);

    // 错误处理, 应该不会跑进来
    if name == &invalid_name {
        println!("Please provide a name for the electronic component.");
        return Err("No name provided".into());
    }
    // 输入数量
    println!("Enter the quantity of the electronic component: ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input = input.trim(); // 去除输入两端的空白字符
    let number = match input.parse::<u32>() {
        Ok(num) => num,
        Err(_) => {
            return Err("Please enter a valid number, must be > 0.".into());
        }
    };
    // 输入描述
    println!("Enter the description of the electronic component: ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let describe = input.trim(); // 去除输入两端的空白字符
                                 // 输入类型
    println!("Enter the type of the electronic component: ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let element_type = input.trim(); // 去除输入两端的空白字符

    let res = Element {
        describe: describe.to_string(),
        model: name.clone(),
        number: number,
        element_type: ElementType::from_string(element_type)?,
        state: ElementStatus::from_number(number)?,
    };
    bom_manage_ctrl.add_element(res)?;
    //
    println!("Adding electronic component: {}", name);
    Ok(())
}

/**
 * @description:  查看电子元件
 * @param {*} matches 命令行参数, 查看元件或者all查看所有
 * @param {*} bom_manage_ctrl
 * @return {*}
 */
fn view_electronic_component(
    matches: &ArgMatches,
    bom_manage_ctrl: &mut BomManageCtrl,
) -> Result<(), Box<dyn Error>> {
    let invalid_name = "null".to_string();
    let name = matches.get_one::<String>("name").unwrap_or(&invalid_name);

    // 错误处理, 应该不会跑进来
    if name == &invalid_name {
        println!("Please provide a name for the electronic component.");
        return Err("No name provided".into());
    }
    if name == SUBCOMMAND_ALL!() {
        for (key, value) in bom_manage_ctrl.element_map.iter() {
            println!("Key: {key}, Value: {:#?}", value);
        }
    } else {
        let element = bom_manage_ctrl.element_map.get(name);
        match element {
            Some(element) => {
                println!("{:#?}", element);
            }
            None => {
                println!("No such electronic component: {}", name);
            }
        }
    }
    Ok(())
}

fn remove_electronic_component(
    matches: &ArgMatches,
    bom_manage_ctrl: &mut BomManageCtrl,
) -> Result<(), Box<dyn Error>> {
    let invalid_name = "null".to_string();
    let name = matches.get_one::<String>("name").unwrap_or(&invalid_name);

    // 错误处理, 应该不会跑进来
    if name == &invalid_name {
        println!("Please provide a name for the electronic component.");
        return Err("No name provided".into());
    }
    
    // name 等于 all, 删除库所有数据
    if name == SUBCOMMAND_ALL!() {
        let mut input = String::new();

        println!("Confirm remove all electronic components? (y/N) default N. ");
        io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
        let input = input.trim(); // 去除输入两端的空白字符

        if input == "y" {
            let _ = bom_manage_ctrl.remove_element(name);
            println!("Remove all electronic components. ");
        }
        return Ok(());
    }
    println!("Enter the reduce quantity of the electronic component, input \"all\" remove this electronic component. ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input = input.trim(); // 去除输入两端的空白字符

    // 移除元件
    if input == SUBCOMMAND_ALL!() {
        match bom_manage_ctrl.remove_element(name) {
            Ok(_) => {}
            Err(_) => {
                println!("No such electronic component: {}", name);
            }
        }
    } else {
        // 减少一定数量元件 输入数量
        let number = match input.parse::<u32>() {
            Ok(num) => num,
            Err(_) => {
                return Err("Please enter a valid number, must be > 0.".into());
            }
        };

        bom_manage_ctrl.reduce_element(name.to_string(), number)?;
    }

    Ok(())
}
/**
 * @description: 处理命令流程
 * @param {Vec} args 命令行参数
 * @return {*}
 */
fn command_handle(args: Vec<String>, bom_manage_ctrl: &mut BomManageCtrl) {
    // 创建一个 clap 的 Command 对象，定义命令行接口
    let matches_result = Command::new("My CLI App")
        .subcommand_required(true)
        .subcommand(
            Command::new("greet")
                .about("Prints a greeting message")
                .arg(
                    Arg::new("name")
                        .help("Name of the person to greet")
                        .required(false)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand(Command::new("status").about("Prints the current status"))
        .subcommand(
            Command::new(COMMAND_ADD!())
                .about("Add a new electronic component")
                .arg(
                    Arg::new("name")
                        .help(
                            "Input the name of the electronic component, such as R10K, C20uF, etc.",
                        )
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand(
            Command::new(COMMAND_REMOVE!())
                .about("Remove a new electronic component")
                .arg(
                    Arg::new("name")
                        .help("Remove a new electronic component, such as R10K, C20uF, all, etc.")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand(
            Command::new(COMMAND_VIEW!())
                .about("View some new electronic component")
                .arg(
                    Arg::new("name")
                        .help("View some new electronic component, such as R10K, C20uF, all, etc.")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand(Command::new(COMMAND_MODIFY!()).about("Modify a new electronic component"))
        .subcommand(Command::new("exit").about("Exit the program"))
        .try_get_matches_from(args);

    // 处理命令行参数
    match matches_result {
        Ok(matches) => match matches.subcommand() {
            Some(("greet", sub_matches)) => handle_greet(sub_matches),
            Some(("status", _sub_matches)) => handle_status(),
            Some((COMMAND_ADD!(), sub_matches)) => {
                match add_electronic_component(sub_matches, bom_manage_ctrl) {
                    Ok(_) => println!("Add electronic component successfully!"),
                    Err(err) => println!("Error: {err}"),
                }
            }
            Some((COMMAND_VIEW!(), sub_matches)) => {
                match view_electronic_component(sub_matches, bom_manage_ctrl) {
                    Ok(_) => {}
                    Err(err) => println!("Error: {err}"),
                }
            }
            // Some((COMMAND_MODIFY!(), sub_matches)) => match modify_electronic_component(sub_matches, bom_manage_ctrl) {

            // },
            Some((COMMAND_REMOVE!(), sub_matches)) => {
                match remove_electronic_component(sub_matches, bom_manage_ctrl) {
                    Ok(_) => {},
                    Err(err) => println!("Error: {err}"),
                }
            }
            Some(("exit", _sub_matches)) => {
                println!("Exiting...");
                process::exit(0);
            }
            _ => println!("Invalid command"),
        },
        Err(err) => {
            println!("{}", err);
        }
    }
}

fn main() {
    let progam_name = env!("CARGO_PKG_NAME");
    let mut bom_manage_ctrl = match BomManageCtrl::new(DATA_FILE!(), TABLE_NAME!()) {
        Ok(bom_manage_ctrl) => bom_manage_ctrl,
        Err(error) => panic!("Error: {error}"),
    };
    let config = Config::builder().build();
    let mut rl = Editor::<(), DefaultHistory>::with_config(config).unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = match rl.readline(format!("{progam_name}>> ").as_str()) {
            Ok(line) => {
                match rl.add_history_entry(line.as_str()) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error: {err}");
                    }
                }
                line
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        // 解析数据
        match get_cmd(&progam_name, readline) {
            Ok(args) => {
                command_handle(args, &mut bom_manage_ctrl);
                rl.save_history("history.txt").unwrap();
            }
            Err(err) => match err.kind() {
                io::ErrorKind::Other => {
                    // 不做处理
                    continue;
                }
                _ => {
                    println!("Error: {err}");
                    continue;
                }
            },
        }
    }
    rl.save_history("history.txt").unwrap();
}

fn handle_greet(matches: &ArgMatches) {
    let binding = "World".to_string();
    let name = matches.get_one::<String>("name").unwrap_or(&binding);
    println!("Hello, {}!", name);
}

fn handle_status() {
    println!("Everything is running smoothly.");
}
