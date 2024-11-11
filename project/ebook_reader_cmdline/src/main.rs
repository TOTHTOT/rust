use env_logger::Builder;
use log::{debug, error, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::sync::mpsc;
use std::{fs, thread};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::terminal_size;

/* 宏定义 */
// 阅读器配置信息报错位置宏
macro_rules! config {
    () => {
        "./config.json"
    };
}
// 终端能显示的字符宽度数量
macro_rules! term_width {
    () => {
        30
    };
}
/* 电子书相关信息 */
#[derive(Serialize, Deserialize, Debug)]
struct BookInfo {
    title: String,
    path: String,
    author: String,
    progress: u64,         // 进度, 和文件指针相关
    filesize: u64,         // 文件大小
    progress_percent: f32, // 进度百分比, 导入书籍使用
    file_avilable: bool,   // 文件是否可用
}

/* 电子书的方法 */
#[allow(dead_code)]
impl BookInfo {
    pub fn new(title: String, author: String, path: String, progress_percent: f32) -> BookInfo {
        // 获取文件大小
        let file_size = fs::metadata(path.clone()).unwrap().len();
        BookInfo {
            title,
            author,
            path,
            progress: file_size * (progress_percent / 100.0) as u64,
            filesize: file_size,
            progress_percent,
            file_avilable: true,
        }
    }

    /**
     * @description: 根据设定的阅读百分比获取文件指针位置
     * 会修改 filesize 和 progress 字段
     * @param {*} mut
     * @return {*}
     */    
    pub fn cal_progress(&mut self) {
        let filesize = fs::metadata(&self.path).unwrap().len();
        self.filesize = filesize;
        let mut file_start_seek = (filesize as f32 * (self.progress_percent / 100.0)) as u64;
        debug!(
            "filesize: {}, progress_percent: {}, initial file_start_seek: {}",
            filesize, self.progress_percent, file_start_seek
        );

        let mut file = fs::File::open(&self.path).unwrap();
        file.seek(io::SeekFrom::Start(file_start_seek)).unwrap();

        let mut buffer = vec![0; 1]; // 单字节缓冲区用于检查逐个字节的 UTF-8 有效性
        let mut utf8_check = Vec::new();
        let mut valid_utf8_found = false;

        while file_start_seek < filesize {
            if file.read_exact(&mut buffer).is_err() {
                break; // 到达文件末尾
            }

            utf8_check.push(buffer[0]);

            // 检查当前积累的字节是否是有效的 UTF-8
            if std::str::from_utf8(&utf8_check).is_ok() {
                valid_utf8_found = true;
                file_start_seek += 1;
                break;
            } else if utf8_check.len() > 4 {
                // 如果积累的字节数超过 4 且不是有效的 UTF-8，清空重试
                utf8_check.clear();
                file_start_seek += 1;
                file.seek(io::SeekFrom::Start(file_start_seek)).unwrap();
            } else {
                file_start_seek += 1;
            }
        }
        info!(
            "utf8_check: {:?}, file_start_seek: {}, len: {}",
            utf8_check,
            file_start_seek,
            utf8_check.len(),
        );

        if !valid_utf8_found {
            file_start_seek = 0; // 如果没找到有效 UTF-8，回退到文件开头
            warn!("Failed to find valid UTF-8 boundary; defaulting to start of file.");
        }

        info!(
            "filesize: {}, final file_start_seek: {}, progress_percent: {}",
            filesize, file_start_seek, self.progress_percent
        );
        // 最终要减去查找的长度, 避免漏显示一个字符
        self.progress = file_start_seek - utf8_check.len() as u64;
    }
}

/* 按键对应事件枚举 */
#[allow(dead_code)]
#[derive(Debug)]
enum EbookReaderHotKeyType {
    NextLine,
    PreviousLine,
    ExitReadMode,
    Unsupport,
}
#[allow(dead_code)]
impl EbookReaderHotKeyType {
    pub fn transform_keytype(key: u8) -> EbookReaderHotKeyType {
        match key {
            0x0a => EbookReaderHotKeyType::NextLine,
            66 => EbookReaderHotKeyType::PreviousLine,
            0x5c => EbookReaderHotKeyType::ExitReadMode,
            _ => EbookReaderHotKeyType::Unsupport,
        }
    }
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
                info!("total books: {}", reader.books.len());

                for book in &reader.books {
                    info!(
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
        let mut progress_percent = String::new();
        io::stdin().read_line(&mut progress_percent).unwrap();
        let mut book = BookInfo {
            title: String::new(),
            author: String::new(),
            path: path.trim().to_string(),
            progress: 0, // 暂时是0 后面统一处理
            filesize: 0,
            progress_percent: progress_percent.trim().parse().unwrap(),
            file_avilable: true,
        };
        // 根据设定的阅读进度转为书籍阅读时的指针偏移地址
        book.cal_progress();

        info!("book is saved {:#?}", book);
        self.books.push(book);

        self.to_json(self.cfg_json_path.as_str()).unwrap();
    }

    /**
     * @description: 打印保存的书籍信息, 会判断文件是否存在, 并标记 file_avilable 属性
     * @param {*} self
     * @return {*}
     */
    pub fn check_save_book(&mut self) {
        let book_len = self.books.len();
        for i in 0..book_len {
            // 检测文件是否存在
            let book = &mut self.books[i];
            book.file_avilable = fs::metadata(&book.path).is_ok();
            println!(
                "[{}]path: {}, progress_percent: {}%, file_avilable: {}",
                i,
                book.path,
                book.progress_percent / 100.0,
                book.file_avilable
            );
            self.to_json(&self.cfg_json_path)
                .expect("save json file failed");
        }
        println!("");
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
     * @description: 读书时的快捷键处理, 阻塞接收, 使用 termion 库
     * 目前监听  PgUp, PgDown, Esc, 通过 通道 传递给主线程处理
     * @return {*}
     */
    pub fn get_input_key(
        tx: &mpsc::Sender<EbookReaderHotKeyType>,
    ) -> Result<EbookReaderHotKeyType, io::Error> {
        let stdin = io::stdin();

        // 让终端进入原始模式, 不然有些按键会被替换成其他字符导致不能正确接收按键输入
        let mut _stdout = io::stdout().into_raw_mode().unwrap();

        for c in stdin.keys() {
            match c.unwrap() {
                termion::event::Key::PageDown => {
                    tx.send(EbookReaderHotKeyType::NextLine).unwrap();
                    return Ok(EbookReaderHotKeyType::NextLine);
                }
                termion::event::Key::PageUp => {
                    tx.send(EbookReaderHotKeyType::PreviousLine).unwrap();
                    return Ok(EbookReaderHotKeyType::PreviousLine);
                }
                termion::event::Key::End => {
                    tx.send(EbookReaderHotKeyType::ExitReadMode).unwrap();
                    return Ok(EbookReaderHotKeyType::ExitReadMode);
                }
                _ => {
                    tx.send(EbookReaderHotKeyType::Unsupport).unwrap();
                    return Err(io::Error::new(io::ErrorKind::Other, "Unsupported key"));
                }
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "No key event"))
    }

    /**
     * @description: 输出一行内容, 根据窗口和缓冲区计算输出内容
     * @return {*}
     */
    fn display_line_segment(
        &mut self,
        line_char: &[char],
        term_width: usize,
        display_window_cnt: &mut usize,
        pre_linelen: &mut usize,
    ) {
        // 清空当前行
        print!("\r{}{}", " ".repeat(*pre_linelen), "\r");
        io::stdout().flush().unwrap(); // 强制刷新输出

        // 计算要输出的内容
        let confirm_show_line = {
            let start_index = term_width * *display_window_cnt;
            let end_index = {
                if line_char.len() > term_width + start_index {
                    term_width * (*display_window_cnt + 1)
                } else {
                    line_char.len()
                }
            };
            line_char[start_index..end_index].iter().collect::<String>()
        };

        *display_window_cnt += 1;

        // 输出一行书籍内容
        print!("{}", confirm_show_line);
        io::stdout().flush().unwrap(); // 强制刷新输出

        *pre_linelen = confirm_show_line.len(); // 更新前一行长度
    }

    /**
     * @description: 读书, 根据书籍的进度读取
     * @param {*} mut
     * @return {*}
     */
    fn read_book(&mut self) {
        // 根据索引选择阅读书籍
        println!("input book index:");
        self.check_save_book();
        let mut index: String = String::new();
        io::stdin().read_line(&mut index).unwrap();
        let book_index: usize = index.trim().parse::<usize>().unwrap();
        if book_index >= self.books.len() {
            println!("book index error");
            return;
        }
        // 会的书籍信息
        let book = &self.books[book_index];

        // 打开书, 从 progress 位置开始读
        let mut file = fs::File::open(&book.path).unwrap();
        println!("open book: {}, progress: {}", book.path, book.progress);
        file.seek(io::SeekFrom::Start(book.progress)).unwrap();
        // 书籍内容加入缓冲区
        let mut book_content = BufReader::new(file);
        let mut pre_linelen = 0;

        // 创建通道实现通信 启动线程监听按键
        let (tx, rx) = mpsc::channel();
        let tx1 = tx.clone();
        let thread = thread::spawn(move || loop {
            match EbookReader::get_input_key(&tx1) {
                Ok(key) => {
                    match key {
                        // 收到退出信号, 其他不处理
                        EbookReaderHotKeyType::ExitReadMode => {
                            break;
                        }
                        _ => {
                            // 其他按键不处理
                        }
                    }
                }
                Err(e) => {
                    error!("get input key error: {}", e);
                    break;
                }
            };
        });

        // 开始读书
        let mut line = String::new();
        let mut current_line_remain = 0; // 当前行已经显示的字符, == 0时才允许读取下一行

        /* 当前行字符, line 转换来的,
        实现逻辑是: 当读取的一行内容大于终端宽度时, 需要分段处理,
        每次现实的内容的窗口在 line_char 中移动, 直到移动到末尾才读取下一行数据.  */
        let mut line_char: Vec<char> = Vec::new();
        let mut display_window_cnt = 0; // 已经显示的窗口数, 程序读一行时清空,显示一行数据时增加
        let (_width, _height) = terminal_size().unwrap();
        // let term_width = width as usize; // 获取终端宽度
        let term_width = term_width!(); // 获取终端宽度
        debug!("term width: {}", term_width);
        loop {
            if current_line_remain <= 0 {
                // 读取下一行时才清空不然在大于 term_width 字符情况下显示 term_width 字符后面内容时 line 没数据
                line.clear();
                if BufRead::read_line(&mut book_content, &mut line).unwrap() == 0 {
                    debug!("read book end");
                    break;
                }
                // 转为字符集合
                line_char = line.trim().chars().collect();
                current_line_remain = 0;
                display_window_cnt = 0;
                current_line_remain += line_char.len();
                debug!("current line_char remain: {current_line_remain}");
            }
            // 减去已经显示了的字符
            current_line_remain = {
                if current_line_remain > term_width {
                    current_line_remain - term_width
                } else {
                    current_line_remain - current_line_remain
                }
            };
            debug!("current line remain: {current_line_remain}");
            let line_tirm = line.trim();
            // 如果获取到的是空行直接下一行
            if line_tirm.is_empty() {
                continue;
            }
            // 显示窗口数据
            self.display_line_segment(
                &line_char,
                term_width,
                &mut display_window_cnt,
                &mut pre_linelen,
            );

            // 阻塞等待按键监听线程发来的消息
            match rx.recv() {
                Ok(EbookReaderHotKeyType::ExitReadMode) => {
                    debug!("recv exit read mode");
                    break;
                }
                Ok(EbookReaderHotKeyType::NextLine) => {
                    // 保存当前阅读进度
                    self.books[book_index].progress = book_content
                        .stream_position()
                        .expect("get book progress error");
                    debug!("save book progress: {}", self.books[book_index].progress);
                    self.books[book_index].progress_percent = self.books[book_index].progress
                        as f32
                        / self.books[book_index].filesize as f32;
                    self.to_json(&self.cfg_json_path)
                        .expect("save book progress error");
                    continue;
                }
                Ok(EbookReaderHotKeyType::PreviousLine) => {
                    continue;
                }
                Ok(EbookReaderHotKeyType::Unsupport) => {
                    error!("unsupport key, exit");
                    break;
                }
                Err(e) => {
                    error!("rx error: {e}");
                    break;
                }
            }
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
                "{} [{}] [{}:{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter(None, LevelFilter::Warn)
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
