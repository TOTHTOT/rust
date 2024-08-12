/*
 * @Description: 电子元件管理程序库
 * @Author: TOTHTOT
 * @Date: 2024-07-05 13:41:11
 * @LastEditTime: 2024-08-12 17:01:49
 * @LastEditors: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @FilePath: \rust\project\bom_manage_lib\src\lib.rs
 */
pub mod bom_manage {
    use rusqlite::{Connection, Result};
    use std::collections::HashMap;
    // use serde_json;
    use std::error::Error;
    use std::{file, fs, io::*, line};

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

    // 数据库表头宏
    macro_rules! insert_into_template {
        () => {
            "INSERT OR IGNORE INTO {} (describe, model, number, element_type, state) VALUES (?, ?, ?, ?, ?)"
        };
    }
    // 读取数据库中所有数据命令
    macro_rules! get_all_template {
        () => {
            "SELECT describe, model, number, element_type, state FROM {}"
        };
    }
    // 读取数据库中所有数据命令
    macro_rules! get_row_num_template {
        () => {
            "SELECT COUNT(*) FROM {}"
        };
    }
    // 读取数据库中所有数据命令, get_row_num_template 失败时执行
    macro_rules! get_row_num_template_fail {
        () => {
            "SELECT 0"
        };
    }
    // 创建表头宏命令
    macro_rules! create_table_template {
        () => {
            "CREATE TABLE IF NOT EXISTS bom_data (id INTEGER PRIMARY KEY, describe TEXT NOT NULL, model TEXT NOT NULL, number INTEGER NOT NULL, element_type INTEGER NOT NULL, state INTEGER NOT NULL)"
        };
    }
    // 删除表内所有内容
    macro_rules! delete_all_template {
        () => {
            "DELETE FROM {}"
        };
    }

    // 元件类别
    #[derive(Debug, Clone)]
    pub enum ElementType {
        Resistor,   // 电阻
        Diode,      // 二极管
        Transistor, // 三极管
        Capacitor,  // 电容
        Inductor,   // 电感
        Unknown,    // 未知
    }
    // ElementType 的方法
    impl ElementType {
        /**
         * @name: to_string
         * @msg: 类型转为字符串
         * @param {*} self
         * @return {*}
         * @author: TOTHTOT
         * @Date: 2024-07-31 09:14:58
         */
        pub fn to_string(&self) -> String {
            match self {
                ElementType::Resistor => "电阻".to_string(),
                ElementType::Diode => "二极管".to_string(),
                ElementType::Transistor => "三极管".to_string(),
                ElementType::Capacitor => "电容".to_string(),
                ElementType::Inductor => "电感".to_string(),
                ElementType::Unknown => "未知".to_string(),
            }
        }

        /**
         * @name: from_string
         * @msg: 字符串转为ElementType类型
         * @param { &str } s 字符串
         * @return {返回ElementType类型, 不支持的类型返回ErrType}
         * @author: TOTHTOT
         * @Date: 2024-07-31 09:15:41
         */
        pub fn from_string(s: &str) -> Result<ElementType, String> {
            match s {
                "电阻" => Ok(ElementType::Resistor),
                "二极管" => Ok(ElementType::Diode),
                "三极管" => Ok(ElementType::Transistor),
                "电容" => Ok(ElementType::Capacitor),
                "电感" => Ok(ElementType::Inductor),
                _ => Err("Error ElementType".to_string()),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum ElementStatus {
        ALOT,     // 丰富
        NORMAL,   // 一般
        SHORTAGE, // 缺货
        Unknown,  // 未知
    }
    // ElementStatus 的方法
    impl ElementStatus {
        /**
         * @name: to_string
         * @msg: 类型转字符串
         * @param {*} self
         * @return {转译后的字符串, 类型错误返回空字符串}
         * @author: TOTHTOT
         * @Date: 2024-07-31 09:22:04
         */
        pub fn to_string(&self) -> String {
            match self {
                ElementStatus::ALOT => "丰富".to_string(),
                ElementStatus::NORMAL => "一般".to_string(),
                ElementStatus::SHORTAGE => "缺货".to_string(),
                ElementStatus::Unknown => "未知".to_string(),
            }
        }

        /**
         * @name: from_string
         * @msg: 字符串转为 ElementStatus 类型, 需要解包
         * @param {*} s
         * @return {Ok(ElementStatus) 匹配的类型; Err(&str) 失败的字符串}
         * @author: TOTHTOT
         * @Date: 2024-07-31 09:25:47
         */
        pub fn from_string(s: &str) -> Result<ElementStatus, String> {
            match s {
                "丰富" => Ok(ElementStatus::ALOT),
                "一般" => Ok(ElementStatus::NORMAL),
                "缺货" => Ok(ElementStatus::SHORTAGE),
                _ => Err("ErrorNumber".to_string()),
            }
        }

        pub fn from_number(num: u32) -> Result<ElementStatus, String> {
            if num > 100 {
                Ok(ElementStatus::ALOT)
            } else if num > 50 && num <= 100 {
                Ok(ElementStatus::NORMAL)
            } else if num > 0 && num <= 50 {
                Ok(ElementStatus::SHORTAGE)
            } else {
                Ok(ElementStatus::Unknown)
            }
        }
    }

    // 元件信息
    #[derive(Debug, Clone)]
    pub struct Element {
        pub describe: String,          // 元件描述
        pub model: String,             // 元件型号
        pub number: u32,               // 元件数量
        pub element_type: ElementType, // 元件类型
        pub state: ElementStatus,      // 元件状态
    }

    // 保存bom信息的数据库信息
    pub struct DataBaseInfo {
        pub filepath: String,
        pub conn: Connection,
        pub tables: String,
    }

    // DataBaseInfo 的方法
    impl DataBaseInfo {
        /**
         * @name: write_hm_to_database
         * @msg: 写入数据到sqlite数据库, 先删除所有数据再写入数据
         * @param {&HashMap<String, Element>} map
         * @param {*} conn 连接的sqlite 数据库
         * @param {*} tables 表名
         * @return {*}
         * @author: TOTHTOT
         * @Date: 2024-07-31 09:48:28
         */
        pub fn write_hm_to_database(
            self: &Self,
            map: &HashMap<String, Element>,
        ) -> Result<(), Box<dyn Error>> {
            if let Err(e) = self
                .conn
                .execute(format!(delete_all_template!(), self.tables).as_str(), [])
            {
                info_log!("Failed to delete data: {}", e);
                return Err("Failed to delete data".into());
            };
            // 将哈希表写入数据库
            for (_key, value) in map.iter() {
                self.conn
                    .execute(
                        format!(insert_into_template!(), self.tables).as_str(),
                        &[
                            &value.describe,
                            &value.model,
                            &value.number.to_string(),
                            &value.element_type.to_string(),
                            &value.state.to_string(),
                        ],
                    )
                    .expect("Failed to insert data");
            }
            Ok(())
        }

        /**
         * @name:read_hm_from_database
         * @msg: 读取数据库内容
         * @param {*} self
         * @return {*}
         * @author: TOTHTOT
         * @Date: 2024-07-31 10:33:07
         */
        pub fn read_hm_from_database(
            self: &Self,
        ) -> Result<HashMap<String, Element>, Box<dyn Error>> {
            // 准备 SQL 查询语句
            let mut stmt = self
                .conn
                .prepare(format!(get_all_template!(), self.tables).as_str())
                .expect("Failed to prepare statement");
            // 执行查询语句，迭代处理每一行结果, data_iter 是个迭代器
            let data_iter = stmt.query_map([], |row| {
                Ok(Element {
                    describe: match row.get(0) {
                        Ok(describe) => {
                            // info_log!("describe: {}", describe);
                            describe
                        }
                        Err(_) => "Unknown".to_string(),
                    },
                    model: row.get(1).expect("Failed to get model"),
                    number: row.get(2).expect("Failed to get number"),
                    // 字符串转枚举
                    element_type: match row.get::<usize, String>(3) {
                        Ok(element_type_str) => ElementType::from_string(&element_type_str)
                            .unwrap_or(ElementType::Unknown),
                        Err(_) => ElementType::Unknown,
                    },
                    // 字符串转枚举
                    state: match row.get::<usize, String>(4) {
                        Ok(element_status_str) => ElementStatus::from_string(&element_status_str)
                            .unwrap_or(ElementStatus::Unknown),
                        Err(err) => {
                            info_log!("Failed to get element status: {}", err);
                            ElementStatus::Unknown
                        }
                    },
                })
            })?; // 为什么可以用?, Box<dyn Error> 什么意思

            let mut map: HashMap<String, Element> = HashMap::new();
            for result in data_iter {
                match result {
                    Ok(element) => {
                        map.insert(element.model.clone(), element);
                    }
                    Err(_) => {
                        info_log!("Failed to get element");
                        continue;
                    }
                };
            }
            Ok(map)
        }
    }

    // bom控制结构体
    pub struct BomManageCtrl {
        pub database: DataBaseInfo,
        pub element_map: HashMap<String, Element>,
    }

    // BomManageCtrl 的方法
    impl BomManageCtrl {
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
        pub fn new<'a>(
            data_filepath: &'a str,
            table_name: &'a str,
        ) -> Result<BomManageCtrl, String> {
            // 判断文件是否存在且数据有效
            match check_datafile(data_filepath) {
                Ok(_) => {
                    // 文件存在且有效, 读取文件内容
                    match open_or_create_data_file(data_filepath) {
                        Ok(content) => {
                            let mut map: HashMap<String, Element> = HashMap::new();
                            // 行数, 根据行数判断是否需要读取数据到哈希表中, 先借用 content 避免所有权问题
                            let count = database_get_line(&content, table_name);
                            let baseinof = DataBaseInfo {
                                conn: content,
                                filepath: data_filepath.to_string(),
                                tables: table_name.to_string(),
                            };

                            if count > 0 {
                                // 读取数据到哈希表
                                map = match baseinof.read_hm_from_database() {
                                    Ok(map) => map,
                                    Err(err) => {
                                        info_log!("{err}");
                                        return Err(format!(
                                            "Error reading from database: {}",
                                            err
                                        ));
                                    }
                                };
                            } else {
                                info_log!("{table_name} 表为空");
                                // test_write_to_database(&baseinof);
                            }
                            Ok(BomManageCtrl {
                                database: baseinof,
                                element_map: map,
                            })
                        }
                        Err(error) => {
                            info_log!("{error}");
                            Err("文件创建失败".to_string())
                        }
                    }
                }
                Err(error) => {
                    // 文件不存在或无效, 创建文件
                    info_log!("{error}");
                    match open_or_create_data_file(data_filepath) {
                        Ok(content) => {
                            let baseinof = DataBaseInfo {
                                conn: content,
                                filepath: data_filepath.to_string(),
                                tables: table_name.to_string(),
                            };
                            let map: HashMap<String, Element> = HashMap::new();

                            Ok(BomManageCtrl {
                                database: baseinof,
                                element_map: map,
                            })
                        }
                        Err(error) => {
                            info_log!("{error}");
                            Err("文件创建失败".to_string())
                        }
                    }
                }
            }
        }

        /**
         * @name: add_element
         * @msg: 添加元件到哈希表中, 如果已经存在则修改数量
         * @param {*} map 哈希表
         * @param {*} element 要写入到表中元件
         * @return {*}
         * @author: TOTHTOT
         * @Date: 2024-08-01 14:40:26
         */
        pub fn add_element(self: &mut Self, mut element: Element) -> Result<(), String> {
            // 根据键获取数据
            match self.element_map.get_mut(element.model.as_str()) {
                Some(e) => {
                    // 已经存在了的元件就修改数量
                    element.modify_number(e.number + element.number);
                }
                None => {}
            }
            self.element_map.insert(element.model.clone(), element);
            self.database
                .write_hm_to_database(&self.element_map)
                .map_err(|_| "写入数据库失败".to_string())?;
            Ok(())
        }
        
        /**
         * @description: 减少元件数量
         * @param {String} model 减少的元件名称
         * @param {u32} number 减少的数量
         * @return {*}
         */        
        pub fn reduce_element(&mut self, model: String, number: u32) -> Result<(), String> {
            // 根据键获取数据
            match self.element_map.get_mut(model.as_str()) {
                Some(e) => {
                    // 已经存在了的元件就修改数量
                    e.modify_number(e.number - number);
                    let element = e.clone();
                    self.element_map.insert(element.model.clone(), element);
                    self.database
                        .write_hm_to_database(&self.element_map)
                        .map_err(|_| "写入数据库失败".to_string())?;
                }
                None => {
                    return Err("元件不存在".to_string());
                }
            }
            Ok(())
        }
        /**
         * @name:
         * @msg:
         * @param {*} map
         * @param {*} Element
         * @param {String} model
         * @return {*}
         * @author: TOTHTOT
         * @Date: 2024-08-01 14:41:42
         */
        pub fn del_element(&mut self, model: String) -> Result<(), String> {
            // 从哈希表中删除元素
            self.element_map.remove(&model);

            // 尝试将更新后的哈希表写入数据库
            self.database
                .write_hm_to_database(&self.element_map)
                .map_err(|_| "写入数据库失败".to_string())?;

            // 如果没有错误，返回 Ok(())
            Ok(())
        }
    }

    /**
     * @name: open_or_create_data_file
     * @msg: 打开或者创建sqlite文件
     * @param {*} filepath
     * @return {成功返回 Connection 对象, 需要解包; 失败返回错误信息}
     * @author: TOTHTOT
     * @Date: 2024-07-30 14:33:14
     */
    fn open_or_create_data_file(filepath: &str) -> Result<Connection, String> {
        match Connection::open(filepath) {
            Ok(file) => {
                // 写入表头
                match file.execute(create_table_template!(), []) {
                    Ok(_) => Ok(file),
                    Err(_) => Err("链接数据库失败".to_string()),
                }
            }
            Err(_) => Err("无法打开/创建文件".to_string()),
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
    fn check_datafile(filepath: &str) -> Result<(), String> {
        if fs::metadata(filepath).is_ok() {
            info_log!("{filepath} 文件存在");
            let mut data_file = match fs::File::open(filepath) {
                Ok(file) => file,
                Err(err) => {
                    info_log!("{err}");
                    return Err("文件打开失败".to_string());
                }
            };

            // 读取前16个字节判断数据库文件是否有效
            let mut buffer = [0; 16];
            if let Err(_) = data_file.read_exact(&mut buffer) {
                info_log!("{filepath} 文件无效");
                return Err("文件无效".to_string());
            }

            // SQLite 文件头的 magic number
            let sqlite_magic_number: [u8; 16] = [
                0x53, 0x51, 0x4c, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20,
                0x33, 0x00,
            ];

            if buffer == sqlite_magic_number {
                info_log!("{filepath} 是数据库文件");
                return Ok(());
            } else {
                info_log!("{filepath} 不是数据库文件");
                return Err("不是数据库文件".to_string());
            }
        } else {
            info_log!("{filepath} 文件不存在");
            return Err("文件不存在".to_string());
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
        let mut stmt = conn
            .prepare(format!(get_row_num_template!(), tables).as_str())
            .unwrap_or_else(|err| {
                info_log!("{err}");
                conn.prepare(get_row_num_template_fail!())
                    .expect("Failed to prepare fallback statement")
            });
        let count: u64 = stmt.query_row([], |row| row.get(0)).unwrap_or_else(|err| {
            info_log!("{err}");
            0
        });

        count
    }

    /**
     * @name: test_write_to_database
     * @msg: 测试函数, 写入测试数据到数据库中
     * @param {*} conn 连接的sqlite 数据库
     * @param {*} tables 表名
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-07-31 09:42:17
     */
    #[allow(dead_code)]
    fn test_write_to_database(database: &DataBaseInfo) -> Result<(), Box<dyn Error>> {
        let mut map: HashMap<String, Element> = HashMap::new();
        let element = Element {
            describe: "Component A".to_string(),
            model: "R10K".to_string(),
            number: 10,
            element_type: ElementType::Resistor,
            state: ElementStatus::ALOT,
        };
        let element_2 = Element {
            describe: "Component B".to_string(),
            model: "C20uF".to_string(),
            number: 20,
            element_type: ElementType::Capacitor,
            state: ElementStatus::ALOT,
        };
        // 将实例写入哈希表
        map.insert("component_a".to_string(), element);
        map.insert("component_b".to_string(), element_2);

        // 将哈希表写入数据库
        database.write_hm_to_database(&map)?;

        Ok(())
    }

    /**
     * @name: test_read_from_database
     * @msg: 测试从数据库中读取数据
     * @param {*} database
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-07-31 14:31:01
     */
    #[allow(dead_code)]
    fn test_read_from_database(
        database: &DataBaseInfo,
    ) -> Result<HashMap<String, Element>, String> {
        // 从数据库中读取数据
        let map: HashMap<String, Element> = match database.read_hm_from_database() {
            Ok(map) => map,
            Err(err) => {
                info_log!("{err}");
                return Err(format!("Error reading from database: {}", err));
            }
        };

        // 打印读取的数据
        for (key, value) in &map {
            info_log!("Key: {}, Value: {:#?}", key, value);
        }
        return Ok(map);
    }

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
        pub fn modify_describe(&mut self, describe: String) -> &mut Self {
            self.describe = describe;
            self
        }
        /**
         * @name: modify_number
         * @msg: 修改元件数量
         * @param {*} mut self
         * @param {u32} number 数量
         * @return {self}
         * @author: TOTHTOT
         * @Date: 2024-07-29 10:23:12
         */
        pub fn modify_number(&mut self, number: u32) -> &mut Self {
            self.number = number;
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bom_manage::*;
    use std::error::Error;

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        // 数据文件地址
        const DATA_FILE: &str = "data_resource.db";
        // 表名
        const TABLE_NAME: &str = "bom_data";
        // 创建数据库实例
        let mut bom_manage_ctrl = match BomManageCtrl::new(DATA_FILE, TABLE_NAME) {
            Ok(bom_manage_ctrl) => bom_manage_ctrl,
            Err(error) => panic!("Error: {error}"),
        };

        // 遍历初始化时的哈希表
        println!("初始化哈希表：");
        for (key, value) in bom_manage_ctrl.element_map.iter() {
            println!("Key: {key}, Value: {:#?}", value);
        }

        // 测试增加元件
        let res = Element {
            describe: "电阻".to_string(),
            model: "R20K".to_string(),
            number: 100,
            element_type: ElementType::Resistor,
            state: ElementStatus::ALOT,
        };
        let cap = Element {
            describe: "电容".to_string(),
            model: "C10uF".to_string(),
            number: 100,
            element_type: ElementType::Capacitor,
            state: ElementStatus::ALOT,
        };
        let cap2 = Element {
            describe: "电容".to_string(),
            model: "C10uF".to_string(),
            number: 120,
            element_type: ElementType::Capacitor,
            state: ElementStatus::ALOT,
        };

        bom_manage_ctrl.add_element(res)?;
        bom_manage_ctrl.add_element(cap)?;
        bom_manage_ctrl.add_element(cap2)?;

        println!("测试增加元件成功");
        // 遍历初始化时的哈希表
        for (key, value) in bom_manage_ctrl.element_map.iter() {
            println!("Key: {key}, Value: {:#?}", value);
        }

        bom_manage_ctrl.del_element("C10uF".to_string())?;
        println!("测试删除元件成功");
        for (key, value) in bom_manage_ctrl.element_map.iter() {
            println!("Key: {key}, Value: {:#?}", value);
        }

        Ok(())
    }
}
