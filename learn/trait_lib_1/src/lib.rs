pub trait Summary {
    // 默认实现
    fn summarize(&self) -> String{
        String::from("(Read more...)")
    }
}

// 让结构体支持println!输出
#[derive(Debug)]
pub struct NewsArchive {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

impl Summary for NewsArchive {
    // 如果没有实现的话就使用默认的方式
    // fn summarize(&self) -> String {
    //     format!("{}, by {} ({})", self.headline, self.author, self.location)
    // }
}

impl NewsArchive {
    pub fn get_headline(&self) -> &str {
        &self.headline
    }
}


#[derive(Debug)]
pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

impl Summary for Tweet {
    // 默认实现被覆盖
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_newsarchive() {
        let news = NewsArchive {
            headline: String::from("Penguins win the Stanley Cup Championship!"),
            location: String::from("Pittsburgh, PA, USA"),
            author: String::from("Iceburgh"),
            content: String::from("The Pittsburgh Penguins once again are the best."),
        };
        println!("{}", news.summarize());
    }

    fn test_tweet() {
        let tweet = Tweet {
            username: String::from("horse_ebooks"),
            content: String::from("of course, as you probably already know, people"),
            reply: false,
            retweet: false,
        };
        println!("{}", tweet.summarize());
    }
}
