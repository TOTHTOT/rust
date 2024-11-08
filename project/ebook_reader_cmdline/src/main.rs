use env_logger::Builder;
use log::{debug, error, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::sync::{mpsc};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, thread};

/* 宏定义 */
// 阅读器配置信息报错位置宏
macro_rules! config {
    () => {
        "./config.json"
    };
}
/* 电子书相关信息 */
#[derive(Serialize, Deserialize, Debug)]
struct BookInfo {
    title: String,
    path: String,
    author: String,
    progress: f32,
}

/* 电子书的方法 */
#[allow(dead_code)]
impl BookInfo {
    pub fn new(title: String, author: String, path: String, progress: f32) -> BookInfo {
        BookInfo {
            title,
            author,
            path,
            progress,
        }
    }
}

/* 按键对应事件枚举 */
#[allow(dead_code)]
enum EbookReaderHotKeyType {
    NextLine,
    PreviousLine,
    ExitReadMode,
    Unsupport,
}

/* 阅读器工作页面 */
#[derive(Serialize, Deserialize, Debug)]
enum EbookReaderWorkPage {
    MainPage,
    ReadBookPage,
}
/* 电子书阅读器 */
#[derive(Serialize, Deserialize, Debug)]
struct EbookReader {
    about_soft: String,          // 软件信息, 使用方法
    cfg_json_path: String,       // 配置文件路径
    menu: BTreeMap<i32, String>, // 菜单
    books: Vec<BookInfo>,
    read_book_flag: bool,
    workpage: EbookReaderWorkPage,
}

/* 电子书阅读器的方法 */
impl EbookReader {
    pub fn new(config_path: &str) -> Result<EbookReader, io::Error> {
        // 检测配置文件是否存在
        match EbookReader::check_config(config_path) {
            Ok(reader) => {
                println!("total books: {}", reader.books.len());

                for book in &reader.books {
                    println!(
                        "title: {}, author: {}, path: {}, progress: {}",
                        book.title, book.author, book.path, book.progress
                    );
                }

                Ok(reader)
            }
            Err(e) => {
                println!("Error: {}", e);
                Err(e)
            }
        }
    }

    /**
     * @description: 将 EBookReader 转换为 JSON 文件
     * @param {*} self
     * @param {*} filename
     * @return {*}
     */
    fn to_json(&self, filename: &str) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(filename)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    /**
     * @description: 从 JSON 文件读取 EBookReader
     * @param {*} filename
     * @return {*}
     */
    fn from_json(filename: &str) -> io::Result<EbookReader> {
        let data = fs::read_to_string(filename)?;
        let reader: EbookReader = serde_json::from_str(&data)?;
        Ok(reader)
    }

    /**
     * @description: 检测配置文件是否存在, 不存在就创建, 存在就返回文件句柄
     * @param {*} config_path
     * @return {*}
     */
    fn check_config(config_path: &str) -> io::Result<EbookReader> {
        match EbookReader::from_json(&config_path) {
            Ok(reader) => Ok(reader),
            Err(_) => {
                // 这里文件读取失败了, 说明文件不存在, 需要创建
                let mut reader: EbookReader = EbookReader {
                    about_soft: String::from("Ebook Reader v1.0"),
                    cfg_json_path: String::from(config_path),
                    menu: BTreeMap::new(),
                    books: Vec::new(),
                    read_book_flag: false,
                    workpage: EbookReaderWorkPage::MainPage,
                };
                // 菜单
                reader.menu.insert(0, "check book".to_string());
                reader.menu.insert(1, "add book".to_string());
                reader.menu.insert(2, "delete book".to_string());
                reader.menu.insert(3, "read book".to_string());
                reader.menu.insert(4, "exit".to_string());
                reader.to_json(config_path)?;

                // 启动按键监听线程处理事件

                Ok(reader)
            }
        }
    }

    /**
     * @description: 添加书籍
     * @param {*} mut
     * @return {*}
     */
    pub fn add_book(&mut self) {
        println!("please input book path:");
        let mut path = String::new();
        io::stdin().read_line(&mut path).unwrap();

        println!("please input book progress:");
        let mut progress = String::new();
        io::stdin().read_line(&mut progress).unwrap();

        let book = BookInfo {
            title: String::new(),
            author: String::new(),
            path: path.trim().to_string(),
            progress: progress.trim().parse().unwrap(),
        };

        println!("book is saved {:?}", book);
        self.books.push(book);

        self.to_json(self.cfg_json_path.as_str()).unwrap();
    }

    /**
     * @description: 打印保存的书籍信息
     * @param {*} self
     * @return {*}
     */
    pub fn check_save_book(&self) {
        let mut i = 0;
        for book in &self.books {
            println!("[{}]path: {}, progress: {}", i, book.path, book.progress);
            i += 1;
        }
    }

    /**
     * @description: 打印菜单
     * @param {*} self
     * @return {*}
     */
    fn show_menu(&self) {
        for menu in &self.menu {
            print!("[{}]: {}\n", menu.0, menu.1);
        }
    }

    /**
     * @description: 读书时的快捷键处理
     * @param {*} mut
     * @param {*} inkey 输入按键原始值
     * @return {*}
     */
    pub fn get_input_key() -> Result<(), io::Error> {
        // 去除无效字符, 包含回车
        let key = String::new();
        let mut key_buf = [0; 1];
        io::stdin().read(&mut key_buf).unwrap();
        println!("press key{}", key_buf[0]);
        match key {
            // "\\" => {
            //     // 退出
            //     self.to_json(self.cfg_json_path.as_str()).unwrap();
            // },
            _ => {
                if key.is_empty() {
                    // 空格键
                } else {
                    println!("hotkey: {}", key);
                }
            }
        }
        Ok(())
    }
    /**
     * @description: 读书, 根据书籍的进度读取
     * @param {*} mut
     * @return {*}
     */
    fn read_book(&mut self) {
        println!("input book index:");
        self.check_save_book();

        let mut index: String = String::new();
        io::stdin().read_line(&mut index).unwrap();

        let book_index = index.trim().parse::<usize>().unwrap();
        if book_index >= self.books.len() {
            println!("book index error");
            return;
        }

        let book = &self.books[book_index];

        let mut file = fs::File::open(&book.path).unwrap();
        // 打开书, 从 progress 位置开始读
        println!("open book: {}, progress: {}", book.path, book.progress);
        file.seek(io::SeekFrom::Start((book.progress * 100.0) as u64))
            .unwrap();

        let book_content = BufReader::new(file);
        let mut pre_linelen = 0;

        // 创建通道实现通信 启动线程监听按键
        let (tx, rx) = mpsc::channel();
        let tx1 = tx.clone();
        let thread = thread::spawn(move || {
            let mut times = 0;
            loop {
                if times > 10 {
                    tx1.send(EbookReaderHotKeyType::ExitReadMode).unwrap();
                    break;
                }
                tx1.send(EbookReaderHotKeyType::NextLine).unwrap();
                times += 1;
            }
        });

        // 开始读书
        for lines in book_content.lines() {
            match lines {
                Ok(line) => {
                    // 如果获取到的是空行直接下一行
                    if line.is_empty() {
                        continue;
                    }
                    print!("\r{}{}", " ".repeat(pre_linelen), "\r"); // 清空当前行
                    io::stdout().flush().unwrap(); // 强制刷新输出

                    print!("{}", line.trim()); // 输出新的一行
                    io::stdout().flush().unwrap(); // 强制刷新输出
                    pre_linelen = line.len();
                    match rx.recv() {
                        Ok(EbookReaderHotKeyType::ExitReadMode) => {
                            debug!("recv exit read mode");
                            break;
                        }
                        Ok(EbookReaderHotKeyType::NextLine) => {
                            continue;
                        }
                        Ok(EbookReaderHotKeyType::PreviousLine) => {
                            continue;
                        }
                        Ok(EbookReaderHotKeyType::Unsupport) => {
                            debug!("unsupport key, exit");
                            break;
                        }
                        Err(e) => {
                            error!("rx error: {e}");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("read book error: {}", e);
                }
            };
        }

        thread.join().unwrap();
    }

    pub fn run(&mut self) -> i32 {
        self.show_menu();
        let mut ret = 0;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim().parse::<i32>() {
            Ok(menu_num) => match menu_num {
                0 => {
                    self.check_save_book();
                }
                1 => {
                    self.add_book();
                }
                2 => {
                    debug!("delete book");
                }
                3 => {
                    self.read_book();
                }
                4 => {
                    ret = 1;
                }
                _ => {
                    warn!("menu_num not supported!");
                    self.show_menu();
                }
            },
            _ => {
                warn!("please input menu number!");
            }
        }
        return ret;
    }
}

fn main() -> Result<(), io::Error> {
    // 自定义日志格式
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [{}:{}] [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.target(), // 获取模块路径，通常可表示函数名
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();
    let mut book_reader: EbookReader = EbookReader::new(&config!())?;
    debug!("start ebook reader");
    loop {
        if book_reader.run() == 1 {
            break;
        }
    }
    Ok(())
}
