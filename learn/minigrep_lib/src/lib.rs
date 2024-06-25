/*
 * @Description: 
 * @Author: TOTHTOT
 * @Date: 2024-06-25 11:04:00
 * @LastEditTime: 2024-06-25 15:33:52
 * @LastEditors: TOTHTOT
 * @FilePath: \rust\learn\minigrep_lib\src\lib.rs
 */

pub mod minigrep{
    use std::fs;
    use std::error::Error;

    #[derive(Debug)]
    pub struct Config{
        pub query: String,
        pub filename: String,
    }
    
    impl Config{
        pub fn new(args: &[String]) -> Result<Config, &str>{
            if args.len() < 3 || args.len() > 3{
                return Err("not enough arguments or too many arguments");
            }
            let query = args[1].clone();
            let filename = args[2].clone();

            Ok(Config{query, filename})
        }
        /**
         * @name: run
         * @msg: 读取文件内容
         * @param {*} self
         * @return Error: read_to_string() 返回值
         * @author: TOTHTOT
         * @Date: 2024-06-25 13:52:30
         */
        pub fn run(&self) -> Result<String, Box<dyn Error>>{
            let file_content = fs::read_to_string(&self.filename)?;
            // println!("file contents: \n{}", file_content);
            let search_result = self.search(&self.query, &file_content);
            println!("search result: \n{:?}", search_result);
            // 手动返回错误 测试
            // Err("Some error occurred".into())
            Ok(file_content)
        }

        fn search<'a>(&self,query: &str, contents: &'a str) -> Vec<&'a str>{
            let mut results = Vec::new();

            for line in contents.lines(){
                if line.contains(query){
                    // println!("{}", line);
                    results.push(line);
                }
            }
            results
        }
    }
}