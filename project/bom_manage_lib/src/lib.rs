/*
 * @Description: 
 * @Author: TOTHTOT
 * @Date: 2024-07-05 13:41:11
 * @LastEditTime: 2024-07-30 18:07:16
 * @LastEditors: TOTHTOT
 * @FilePath: \rust\project\bom_manage_lib\src\lib.rs
 */
pub mod bom_manage {
    use std::collections::HashMap;
    use rusqlite::{Connection, Result};
    // use serde_json;
    use std::{fs, line, file, io::*};
    
    // 元件类别
    #[derive(Debug)]
    pub enum ElementType {
        Resistor, // 电阻
        Diode, // 二极管
        Transistor, // 三极管
        Capacitor, // 电容
        Inductor, // 电感
    }
    #[derive(Debug)]
    pub enum ElementStatus {
        ALOT, // 丰富
        NORMAL, // 一般
        SHORTAGE, // 缺货
    }
    // 元件信息
    #[derive(Debug)]
    pub struct Element {
        pub describe: String, // 元件描述
        pub model: String, // 元件型号
        pub number: u32, // 元件数量
        pub element_type: ElementType, // 元件类型
        pub state: ElementStatus, // 元件状态
    }

    // 保存bom信息的数据库信息
    pub struct DataBaseInfo {
        pub filepath: String,
        pub conn: Connection,
    }

    /**
     * @name: info_log
     * @msg: 模块内部打印日志接口
     * @param {String} str
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-07-30 15:21:34
     */    

    macro_rules! info_log {
        ($($arg:tt)*) => {
            println!("{}: {}, {}: {}", module_path!(), file!(), line!(), format_args!($($arg)*));
        };
    }
    /**
     * @name: open_or_create_data_file
     * @msg: 
     * @param {*} filepath
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-07-30 14:33:14
     */        
    fn open_or_create_data_file(filepath:&str) -> Result<Connection, &str> {
        match Connection::open(filepath) {
            Ok(file) => {
                match file.execute("CREATE TABLE IF NOT EXISTS bom_data (
                        id INTEGER PRIMARY KEY,
                        describe TEXT NOT NULL,
                        model TEXT NOT NULL,
                        number INTEGER NOT NULL,
                        element_type INTEGER NOT NULL,
                        state INTEGER NOT NULL)", []) {
                    Ok(_) => {
                        Ok(file)
                    },
                    Err(_) => {
                        Err("链接数据库失败")
                    }
                
                }
            },
            Err(_) => {
                Err("无法打开/创建文件")
            }
        }
    }
    
    /**
     * @name: check_datafile
     * @msg: 检测文件是否为合法sql文件, 合法就返回Ok(_), 否则返回错误信息
     * @param {*} filepath 文件地址
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-07-29 13:57:45
     */        
    fn check_datafile(filepath: &str) -> Result<(), &str> {
        if fs::metadata(filepath).is_ok() {
            info_log!("{filepath} 文件存在");
            let mut data_file = match fs::File::open(filepath) {
                Ok(file) => file,
                Err(err) => {
                    info_log!("{err}");
                    return Err("文件打开失败");
                }
            };

            // 读取前16个字节判断数据库文件是否有效
            let mut buffer = [0; 16];
            if let Err(_) = data_file.read_exact(&mut buffer) {
                info_log!("{filepath} 文件无效");
                return Err("文件无效");
            }

            // SQLite 文件头的 magic number
            let sqlite_magic_number: [u8; 16] = [
                0x53, 0x51, 0x4c, 0x69, 0x74, 0x65, 0x20, 0x66,
                0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20, 0x33, 0x00,
            ];

            if buffer == sqlite_magic_number {
                info_log!("{filepath} 是数据库文件");
                return Ok(());
            }
            else {
                info_log!("{filepath} 不是数据库文件");
                return Err("不是数据库文件");
            }
        }
        else {
            info_log!("{filepath} 文件不存在");
            return Err("文件不存在");
        }
    }

    /**
     * @name: database_get_line
     * @msg: 获取数据库中指定表中的行数
     * @param {*} conn sqlite数据库连接
     * @param {*} tables 表名
     * @return { 返回行数 }
     * @author: TOTHTOT
     * @Date: 2024-07-30 16:51:00
     */    
    fn database_get_line(conn: &Connection, tables: &str) -> u64 {
        let mut stmt = conn.prepare(format!("SELECT COUNT(*) FROM {tables}").as_str()).unwrap_or_else(|err| {
            info_log!("{err}");
            conn.prepare("SELECT 0").expect("Failed to prepare fallback statement")
        });
        let count: u64 = stmt.query_row([], |row| row.get(0)).unwrap_or_else(|err| {
            info_log!("{err}");
            0
        });
        
        count
    }
    
    fn test_write_to_database(conn: &Connection, tables: &str) {
        let mut map:HashMap<String, Element> = HashMap::new();
        let element = Element {
            describe: "Component A".to_string(),
            model: "Model X".to_string(),
            number: 10,
            element_type: ElementType::Resistor,
            state: ElementStatus::ALOT,
        };
    
        // 将实例写入哈希表
        map.insert("component_a".to_string(), element);

        // 将哈希表写入数据库
        for (key, value) in map.iter() {
            conn.execute(format!("INSERT INTO {tables} (describe, model, number, element_type, state) 
                                VALUES (?, ?, ?, ?, ?, ?)").as_str(), 
                                &[
                                    &element.describe, 
                                    &element.model, 
                                    &element.number.to_string(), 
                                    &element.element_type, 
                                    &element.state],).expect("Failed to insert data");
        }
    }

    /**
     * @name: new
     * @msg: 创建时判断是否有数据文件, 
     * 1. 有的话就读取并创建哈希表, 将数据写入, 
     * 2. 没数据文件就创建哈希表等待写入数据到表中.
     * @param {&'a str} data_filepath 数据库地址
     * @param {&'a str} table_name 数据库表名
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-07-29 11:11:17
     */
    pub fn new<'a>(data_filepath: &'a str, table_name: &'a str) -> Result<(DataBaseInfo, HashMap<String, Element>), &'a str> { 
        // 判断文件是否存在且数据有效
        match check_datafile(data_filepath) {
            Ok(_) => { // 文件存在且有效, 读取文件内容
                match open_or_create_data_file(data_filepath) {
                    Ok(content) => {
                        let mut map:HashMap<String, Element> = HashMap::new();
                        let count = database_get_line(&content, table_name);
                        if count > 0 {
                            // 读取数据到哈希表
                        }
                        else {
                            info_log!("{table_name} 表为空");
                            test_write_to_database(&content, table_name);
                        }
                        
                        let baseinof = DataBaseInfo {
                            conn: content,
                            filepath: data_filepath.to_string(),
                        };
                        Ok((baseinof, map))
                    },
                    Err(error) => {
                        info_log!("{error}");
                        Err("文件创建失败")
                    }
                }
            },
            Err(error) => { // 文件不存在或无效, 创建文件
                info_log!("{error}");
                match open_or_create_data_file(data_filepath) {
                    Ok(content) => {
                        let baseinof = DataBaseInfo {
                            conn: content,
                            filepath: data_filepath.to_string(),
                        };
                        Ok((baseinof, HashMap::new()))
                    },
                    Err(error) => {
                        info_log!("{error}");
                        Err("文件创建失败")
                    }
                }
            }
        }
    }

    // pub fn save_to_datafile(data_filepath: &str, element_map: &HashMap<String, Element>) -> Result<(), &str> {
    //     let datafile = Self::open_or_create_data_file(data_filepath).unwrap_or_else(|_| Err("文件创建失败"));
    //     datafile.write_all(&element_map).unwrap_or_else(|_| Err("写入失败"));
    //     Ok(())
    // }

    // Element 结构体的方法
    impl Element {
         /**
         * @name: modify_describe
         * @msg: 修改描述
         * @param {*} mut self
         * @param {String} describe 描述信息
         * @return {void}
         * @author: TOTHTOT
         * @Date: 2024-07-29 10:22:18
         */        
        pub fn modify_describe(&mut self, describe: String) -> &mut Self { self.describe = describe; self}
        /**
         * @name: modify_number
         * @msg: 修改元件数量
         * @param {*} mut self
         * @param {u32} number 数量
         * @return {self}
         * @author: TOTHTOT
         * @Date: 2024-07-29 10:23:12
         */        
        pub fn modify_number(&mut self, number: u32) -> &mut Self { self.number = number; self}

        // 添加一个元件到哈希表
        // pub fn add_element(&mut self, element_map: &mut HashMap<String, Element>) { element_map.insert(self.model.clone(), self.clone()); }
    }

}

#[cfg(test)]
mod tests {
    use bom_manage::*;
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        // 数据文件地址
        const DATA_FILE: &str = "data_resource.db";
        const TABLE_NAME: &str = "bom_data";
        let mut element_map: HashMap<String, Element>;
        let mut databaseinfo: DataBaseInfo;
        match bom_manage::new(DATA_FILE, TABLE_NAME) {
            Ok((my_databaseinfo, my_element_map)) => {
                databaseinfo = my_databaseinfo;
                element_map = my_element_map;
            },
            Err(error) => panic!("Error: {error}"),
        };

        let res = Element{
            describe :"电阻".to_string(),
            model :"R10K".to_string(),
            number : 100,
            element_type : ElementType::Resistor,
            state : ElementStatus::ALOT,
        };
        
        let cap = Element{
            describe :"电容".to_string(),
            model :"C10uF".to_string(),
            number : 100,
            element_type : ElementType::Capacitor,
            state : ElementStatus::ALOT,
        };
        element_map.insert(res.model.clone(), res);
        element_map.insert(cap.model.clone(), cap);

        // 输出内容
        for (model, element) in element_map.iter() {
            println!("{}: {}", model, element.describe);
            println!("{}: {}", model, element.number);
            println!("{}: {:?}", model, element.element_type);
            println!("{}: {:?}", model, element.state);
        }
    }
}
