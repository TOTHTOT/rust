use env_logger::Builder;
use log::{debug, error, info, trace, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::SeekFrom;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::sync::mpsc;
use std::{fs, thread};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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

// 菜单枚举
#[repr(i32)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum EbookMenuFuncType {
    CheckBook = 0,
    AddBook,
    DeleteBook,
    ConfigBook,
    ReadBook,
    Exit,
    Unsupport,
}
//
impl EbookMenuFuncType {
    /**
     * @description: 输入菜单编号转为对应功能类型
     * @param {usize} num 菜单编号
     * @return {*}
     */
    pub fn from_number(num: i32) -> Self {
        match num {
            0 => EbookMenuFuncType::CheckBook,
            1 => EbookMenuFuncType::AddBook,
            2 => EbookMenuFuncType::DeleteBook,
            3 => EbookMenuFuncType::ConfigBook,
            4 => EbookMenuFuncType::ReadBook,
            5 => EbookMenuFuncType::Exit,
            _ => {
                error!("Unsupport menu number: {}", num);
                EbookMenuFuncType::Unsupport
            }
        }
    }
    pub fn to_number(&self) -> i32 {
        *self as i32
    }
}

#[derive(Serialize, Debug)]
struct BookCtrl {
    // 缓冲区
    #[serde(skip)]
    book_content: BufReader<fs::File>,
    // 上一次读取的行长度
    pre_linelen: usize,

    // 一行内容, 原始数据 包含换行符等
    raw_line_content: String,
    /* 当前行内容 raw_line_content 得来,
    保证都是 utf-8 字符, 去除换行符等内容, raw_line_content 转换来的,
    实现逻辑是: 当读取的一行内容大于终端宽度时, 需要分段处理,
    每次现实的内容的窗口在 line_content 中移动, 直到移动到末尾才读取下一行数据.  */
    line_content: Vec<char>,
    // 当前行剩余字符数量
    current_line_remain: usize,
    // 已经显示的窗口数, 程序读一行时清空,显示一行数据时增加
    display_window_cnt: usize,
    // 终端宽度
    term_width: usize,
    // 上一行标志, 如果移动到行头 == true, 此时再次按下上一行就移动文件指针
    at_line_start: bool,
}
// 需要实现 Default 不然 BookInfo 会报错 Deserialize
impl Default for BookCtrl {
    fn default() -> Self {
        // 使用一个 `File` 的空实例或 `io::empty()` 占位
        let dummy_file =
            File::open("/dev/null").unwrap_or_else(|_| File::create("/dev/null").unwrap());
        Self {
            book_content: BufReader::new(dummy_file),
            pre_linelen: 0,
            raw_line_content: String::new(),
            line_content: Vec::new(),
            current_line_remain: 0,
            display_window_cnt: 0,
            term_width: term_width!(),
            at_line_start: false,
        }
    }
}
/* BookCtrl 的方法 */
impl BookCtrl {
    pub fn new(filepath: &str, term_width: usize, progress: u64) -> Self {
        let mut book_file = BookCtrl::open_book(filepath).unwrap();
        // 设置阅读进度
        book_file.seek(io::SeekFrom::Start(progress)).unwrap();
        BookCtrl {
            book_content: BufReader::new(book_file),
            pre_linelen: 0,
            raw_line_content: String::new(),
            line_content: Vec::new(),
            current_line_remain: 0,
            display_window_cnt: 0,
            term_width: term_width,
            at_line_start: false,
        }
    }

    fn open_book(filepath: &str) -> Result<fs::File, io::Error> {
        // 打开文件
        fs::File::open(filepath)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("file open fail: {}", e)))
    }

    /**
     * @description: 读取一行原始数据, 并初始化一些参数
     * @param {*} mut
     * @return {*}
     */
    fn read_line_raw(&mut self) -> Result<(), io::Error> {
        match BufRead::read_line(&mut self.book_content, &mut self.raw_line_content) {
            Ok(_) => {
                // debug!("read line: {}", self.raw_line_content);
                // 需要判断是否是空行, 空行的话要再次
                if self.raw_line_content.trim().len() != 0 {
                    // 读取成功的一些参数初始化
                    self.line_content.clear();
                    self.line_content = self.raw_line_content.trim().chars().collect();

                    self.current_line_remain = self.line_content.len();
                    self.display_window_cnt = 0;
                    self.at_line_start = false;
                    debug!("current line_char remain: {}", self.current_line_remain);

                    return Ok(());
                } else {
                    self.raw_line_content.clear();
                    return Err(io::Error::new(io::ErrorKind::Other, "empty line"));
                }
            }
            Err(e) => {
                error!("read line fail: {}", e);
                return Err(e);
            }
        };
    }

    /**
     * @description: 读取一行数据, 刷新缓冲区
     * @param {*} mut
     * @param {*} read_dir -1 上一行 1 下一行, 0 刷新当前行
     * @return {*}
     */
    fn read_line(&mut self, read_dir: i32) -> Result<(), io::Error> {
        self.raw_line_content.clear();
        if read_dir == 1 {
            loop {
                match self.read_line_raw() {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::Other && e.to_string() == "empty line" {
                            // 如果是空行, 继续读取
                            continue;
                        } else {
                            error!("read line fail: {}", e);
                            return Err(e);
                        }
                    }
                }
            }
        } else if read_dir == -1 {
            // 向后读取一行（上一行）
            let mut new_position = self.book_content.seek(SeekFrom::Current(0)).unwrap();
            if new_position == 0 {
                // 如果在文件开头，无法再回退
                warn!("Already at the beginning of the file.");
                return Ok(());
            }
            trace!("start new_position: {}", new_position);
            loop {
                let mut buffer = vec![];
                let mut char_count = 0;
                let mut find_newline_cnt = 0;

                // 从当前位置向后回溯，读取字节直到遇到换行符或文件开头
                while new_position > 0 {
                    new_position -= 1;
                    self.book_content
                        .seek(SeekFrom::Start(new_position))
                        .unwrap();

                    let mut byte = [0; 1];
                    self.book_content.read_exact(&mut byte).unwrap();

                    if byte[0] == b'\n' && char_count > 0 {
                        trace!("find newline: {}", find_newline_cnt);
                        // 如果已经找到两个以上换行符，说明已经找到了上一行的开始
                        if find_newline_cnt > 1 {
                            break;
                        }
                        find_newline_cnt += 1;
                    }
                    buffer.push(byte[0]);
                    char_count += 1;
                }
                // 将缓冲区反转并转换为字符串
                buffer.reverse();
                let line_str = String::from_utf8_lossy(&buffer).trim().to_string();

                if !line_str.is_empty() {
                    self.read_line_raw()?;
                    return Ok(());
                } else if new_position == 0 {
                    // 到达文件开头且没有找到非空行
                    warn!("Reached the beginning of the file without finding a non-empty line.");
                    return Ok(());
                }
            }
        } else {
            return Ok(());
        }
    }

    /**
     * @description: 根据上一次显示长度清除当前行内容
     * @param {*}
     * @return {*}
     */
    fn clean_line_by_prelen(&self) {
        // 清空当前行
        print!("\r{}{}", " ".repeat(self.pre_linelen), "\r");
        io::stdout().flush().unwrap(); // 强制刷新输出
    }

    /**
     * @description: 根据 display_window_cnt 显示内容
     * @param {*} self
     * @return {*}
     */
    fn show_line_by_term(&mut self) {
        self.clean_line_by_prelen();

        // 根据窗口显示内容
        let confirm_show_line = {
            let start_index = self.term_width * self.display_window_cnt;
            let end_index = {
                if self.line_content.len() > self.term_width + start_index {
                    self.term_width * (self.display_window_cnt + 1)
                } else {
                    self.line_content.len()
                }
            };
            debug!(
                "start_index: {}, end_index: {}\ndisplay_window_cnt: {}, current_line_remain: {}, at_line_start: {}",
                start_index, end_index, self.display_window_cnt, self.current_line_remain, self.at_line_start
            );
            self.line_content[start_index..end_index]
                .iter()
                .collect::<String>()
        };
        self.pre_linelen = confirm_show_line.len();
        // 输出一行书籍内容
        print!("{}", confirm_show_line);
        io::stdout().flush().unwrap(); // 强制刷新输出
    }
    /**
     * @description: 显示下一行, 根据窗口xianshi
     * @param {*} mut
     * @return {*}
     */
    pub fn next_line(&mut self) {
        if self.current_line_remain <= 0 {
            /* 读取下一行时才清空不然在大于 term_width
            字符情况下显示 term_width 字符后面内容时 line 没数据 */
            match self.read_line(1) {
                Ok(_) => {}
                Err(e) => {
                    error!("read line fail: {}", e);
                }
            }
        }
        debug!("current line remain: {}", self.current_line_remain);
        // 减去已经显示了的字符
        self.current_line_remain = {
            if self.current_line_remain > self.term_width {
                self.current_line_remain - self.term_width
            } else {
                self.current_line_remain - self.current_line_remain
            }
        };

        // 显示内容
        self.show_line_by_term();
        self.display_window_cnt += 1;
    }

    /**
     * @description: 查看上一行, 根据窗口显示,
     * 起始索引为0移动文件指针到上一行, 上一行如果长度为0自动再次上一行
     * @param {*} mut
     * @return {*}
     */
    pub fn previous_line(&mut self) {
        trace!("previous line");
        /* 如果当前行内容需要多次显示, 此时上一行移动窗口就好了,
        直到窗口移动到起始位置才会移动文件指针去上一行. */
        if self.display_window_cnt > 1 && self.at_line_start == false {
            trace!("display_window_cnt: {}", self.display_window_cnt);
            self.display_window_cnt -= 2;
            // 设置标志, 确保在行头时的上一行能移动文件指针到上一行
            if self.display_window_cnt == 0 {
                self.at_line_start = true;
            }
            // 加上原先显示的字符
            self.current_line_remain += self.term_width;
            self.show_line_by_term();
            self.display_window_cnt += 1;
        } else {
            match self.read_line(-1) {
                Ok(_) => {}
                Err(e) => {
                    error!("read line fail: {}", e);
                }
            }
            debug!("current line remain: {}", self.current_line_remain);
            // 上一行的 display_window_cnt 特殊处理, 上一行的内容长度大于终端宽度时, display_window_cnt 要为窗口最后一个
            self.display_window_cnt = {
                if self.line_content.len() > self.term_width {
                    self.line_content.len() / self.term_width
                } else {
                    0
                }
            };
            self.current_line_remain = 0;
            // 显示内容
            self.show_line_by_term();
            self.display_window_cnt += 1;
        }
    }

    pub fn close_book(&mut self) {}
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
        let file_size = fs::metadata(&path).unwrap().len();
        BookInfo {
            title,
            author,
            path: path.clone(),
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

    /**
     * @description: 更新阅读进度
     * @param {*} mut
     * @param {BufReader} fbuf 文件缓冲区
     * @return {*}
     */
    pub fn update_progress(&mut self, fbuf: &mut BufReader<fs::File>) {
        self.progress = fbuf.stream_position().expect("get book progress error");
        // debug!("save book progress: {}", self.progress);
        self.progress_percent = self.progress as f32 / self.filesize as f32;
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
    about_soft: String,                        // 软件信息, 使用方法
    cfg_json_path: String,                     // 配置文件路径
    menu: BTreeMap<EbookMenuFuncType, String>, // 菜单
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
                reader
                    .menu
                    .insert(EbookMenuFuncType::CheckBook, "check book".to_string());
                reader
                    .menu
                    .insert(EbookMenuFuncType::AddBook, "add book".to_string());
                reader
                    .menu
                    .insert(EbookMenuFuncType::DeleteBook, "delete book".to_string());
                reader
                    .menu
                    .insert(EbookMenuFuncType::ConfigBook, "config book".to_string());
                reader
                    .menu
                    .insert(EbookMenuFuncType::ReadBook, "read book".to_string());
                reader
                    .menu
                    .insert(EbookMenuFuncType::Exit, "exit".to_string());
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
            print!("[{}]: {}\n", menu.0.to_number(), menu.1);
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
        // 书籍信息
        let book = &self.books[book_index];

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
        let (_width, _height) = termion::terminal_size().unwrap();
        let mut bookctrl = BookCtrl::new(&book.path, 30, book.progress);

        loop {
            // 阻塞等待按键监听线程发来的消息
            match rx.recv() {
                Ok(EbookReaderHotKeyType::ExitReadMode) => {
                    bookctrl.clean_line_by_prelen();
                    debug!("recv exit read mode");
                    break;
                }
                Ok(EbookReaderHotKeyType::NextLine) => {
                    trace!("next line");
                    bookctrl.next_line();
                    // 保存当前阅读进度
                    self.books[book_index].update_progress(&mut bookctrl.book_content);
                    self.to_json(&self.cfg_json_path).unwrap_or_else(|e| {
                        error!("save book progress error: {e}");
                    });
                    continue;
                }
                Ok(EbookReaderHotKeyType::PreviousLine) => {
                    trace!("previous line");
                    bookctrl.previous_line();
                    // 保存当前阅读进度
                    self.books[book_index].update_progress(&mut bookctrl.book_content);
                    self.to_json(&self.cfg_json_path).unwrap_or_else(|e| {
                        error!("save book progress error: {e}");
                    });
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

    fn config_book(&mut self) {
        println!("select book to config");
        self.check_save_book();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("input book index error");

        let book_index = choice.trim().parse::<i32>().unwrap_or_else(|e| {
            error!("parse input key error: {}", e);
            -1
        });
        // 验证选择索引是否正确, 错误的话退出
        if (book_index == -1 || book_index < self.books.len() as i32)
            && self.books[book_index as usize].file_avilable == true
        {
            println!("input book progress");
            let mut progress = String::new();
            match io::stdin().read_line(&mut progress) {
                Ok(_) => {
                    // 验证输入进度是否正确, 错误的话退出
                    let progress_percent = progress.trim().parse::<f32>().unwrap_or_else(|e| {
                        error!("parse input key error: {}", e);
                        -1.0
                    });
                    if progress_percent >= 0.0 && progress_percent <= 100.0 {
                        self.books[book_index as usize].progress_percent = progress_percent;
                        self.books[book_index as usize].cal_progress();
                    } else {
                        error!("progress percent error: {}", progress_percent);
                        return;
                    }
                }
                Err(e) => {
                    error!("get input key error: {}", e);
                    return;
                }
            }
        } else {
            error!("book index error: {}", book_index);
            return;
        }
    }
    pub fn run(&mut self) -> i32 {
        self.show_menu();
        let mut ret = 0;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim().parse::<i32>() {
            Ok(menu_num) => match EbookMenuFuncType::from_number(menu_num) {
                EbookMenuFuncType::CheckBook => {
                    self.check_save_book();
                }
                EbookMenuFuncType::AddBook => {
                    self.add_book();
                }
                EbookMenuFuncType::DeleteBook => {
                    debug!("delete book");
                }
                EbookMenuFuncType::ReadBook => {
                    self.read_book();
                }
                EbookMenuFuncType::Exit => {
                    ret = 1;
                }
                EbookMenuFuncType::ConfigBook => {
                    self.config_book();
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
