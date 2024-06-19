use::std::fs::{File, OpenOptions};
// use std::os::unix::fs::OpenOptionsExt;
use std::io::Write;
use std::io::{self, Read, ErrorKind};

/**
 * @name: read_username
 * @msg: 读取文件数据, ? 传递错误
 * @param {*} file_path 文件地址
 * @return {文件内容, 错误码}
 * @author: TOTHTOT
 * @Date: 2024-06-19 15:59:24
 */
fn read_username(file_path:&str)->Result<String, io::Error>{
    // 打开文件, 只读
    let mut file_open_result = File::open(file_path)?;
    // 读取文件内容
    let mut contents = String::new();
    file_open_result.read_to_string(&mut contents)?;
    
    Ok(contents)
}

fn main() {
    let file_path = "hello.txt";
    // 打开文件, 设置文件权限 读写, 也可以直接.create(true)创建文件
    let open_file_result = OpenOptions::new().read(true).write(true).open(file_path);

    let mut file = match open_file_result{
        Ok(file) => file,
        Err(error) => match error.kind(){
            ErrorKind::NotFound => {
                // 如果文件不存在，创建一个新的文件并写入数据
                println!("File not found, creating a new one");

                let new_file = match OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(file_path)
                {
                    Ok(fc) => fc,
                    Err(e) => panic!("Tried to create file but there was a problem: {:?}", e),
                };
                new_file // 返回新创建的文件句柄
            },
            // 其他错误打印
            other_error => panic!("There was a problem opening the file: {:?}", other_error),
        },
    };

    println!("file stat {:?}", file);
    file.write_all(b"Hello world!\n").unwrap();
    drop(file);

    // 读取文件内容
    let name = read_username(file_path).unwrap();
    println!("name: {}", name);
}
