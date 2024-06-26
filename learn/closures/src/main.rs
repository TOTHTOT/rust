use std::thread;

#[allow(dead_code)] // 忽略未使用的变体
#[derive(Debug, PartialEq, Copy, Clone)]
enum ShirtColor{
    NoneColor,
    Blue,
    Red,
    Green,
    Yellow,
    Pink,
}

struct Inventor{
    shirt:Vec<ShirtColor>,
}

impl Inventor{
    /**
     * @name: getaway
     * @msg: 获取一件客户指定衣服， 如果没有指定就返回库存中数量最多的， 如果没有就返回err
     * @param {*} mut 
     * @param {Option} usr_pref 客户需要的颜色
     * @return {Ok(color) 成功， 返回需要的衣服；Err(str) 失败， 返回失败原因}
     * @author: TOTHTOT
     * @Date: 2024-06-26 16:01:08
     */    
    pub fn getaway(&mut self, usr_pref:Option<ShirtColor>) -> Result<ShirtColor, &str>{
        // 解包如果指定了需要的颜色且有才往下执行, 没有指定颜色就返回库存中数量最多的颜色
        match usr_pref{
            Some(color) =>{
                self.get_select_shirtcolor(color)
            },
            None =>{
                self.get_select_shirtcolor(self.get_most_store())
            }
        }
    }

    /**
     * @name: get_select_shirtcolor
     * @msg: 库存中那周一件衣服
     * @param {*} mut
     * @param {ShirtColor} color
     * @return {Ok(color) 成功， 返回需要的衣服；Err(str) 失败， 返回失败原因}
     * @author: TOTHTOT
     * @Date: 2024-06-26 16:03:34
     */    
    fn get_select_shirtcolor(&mut self, color:ShirtColor)->Result<ShirtColor, &str>{
        if self.store_have_shirtcolor(color){
            for (index, &item) in self.shirt.iter().enumerate(){
                if item == color{
                    self.shirt.remove(index);
                    println!("shirt index {}",index);
                    return Ok(color);
                }
            }
            return Ok(color);
        }else{
            Err("no this color")
        }
    }
    /**
     * @name: store_have_shirtcolor
     * @msg: 是否包含选择的颜色
     * @param {*} self
     * @param {ShirtColor} color
     * @return {*}
     * @author: TOTHTOT
     * @Date: 2024-06-25 16:58:04
     */    
    fn store_have_shirtcolor(&self, color:ShirtColor)->bool{
        self.shirt.contains(&color)
    }
    /**
     * @name: get_most_store
     * @msg: 计算库存中数量最多的颜色
     * @param {*} self
     * @return {返回数量最多的颜色}
     * @author: TOTHTOT
     * @Date: 2024-06-25 16:43:41
     */
    fn get_most_store(&self)->ShirtColor{
        let mut number_shirt_color = [0;4];
        
        for shirt in &self.shirt{
            match shirt{
                ShirtColor::Blue => number_shirt_color[ShirtColor::Blue as usize -1] += 1,
                ShirtColor::Red => number_shirt_color[ShirtColor::Red as usize -1] += 1,
                ShirtColor::Green => number_shirt_color[ShirtColor::Green as usize -1] += 1,
                ShirtColor::Yellow => number_shirt_color[ShirtColor::Yellow as usize - 1] += 1,
                _=>{
                    eprintln!("Error, unknown color");
                    std::process::exit(1);
                },
            }
        }
        println!("store number:{:?}", number_shirt_color);
        let mut max_number = 0;
        let mut give_shirt = ShirtColor::NoneColor;
        for i in 0..number_shirt_color.len(){
            if number_shirt_color[i] > max_number{
                max_number = number_shirt_color[i];
                give_shirt = match i + 1{
                    0 => ShirtColor::NoneColor,
                    1 => ShirtColor::Blue,
                    2 => ShirtColor::Red,
                    3 => ShirtColor::Green,
                    4 => ShirtColor::Yellow,
                    _=>{
                        eprintln!("Error, unknown color");
                        std::process::exit(1);
                    },
                };
            }
        }
        give_shirt
    }
}

fn test_closure() {

    let mut list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");

    let mut borrows_mutably = || list.push(7);

    borrows_mutably();
    println!("After calling closure: {list:?}");

    let list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");

    println!("From main thread: {list:?}");
    // 强制移动所有权到线程
    thread::spawn(move || println!("From thread: {list:?}"))
        .join()
        .unwrap();

    let str = String::from("hello");
    let mut list = vec![];

    println!("Before defining closure: {list:?}");
    println!("str: {str}");
    for i in 0..2 {
        
        list.push(11);
    }
    println!("After defining closure: {list:?}");
    // 所有权被转移 此时str无法使用
    // println!("str: {str}");
    
    // 测试借用
    let str = "hello".to_string();
    let str_1 = &str;

    println!("str: {str}");
    println!("str_1: {str_1}");
    let str_2 = str;
    // println!("str_1: {str_1}"); // 所有权转移了 不能再使用str_1和str
    // println!("str: {str}");
    println!("str_2: {str_2}");
}

fn main() {
    let mut store = Inventor{
        shirt:vec![ShirtColor::Yellow, ShirtColor::Red, ShirtColor::Green, ShirtColor::Yellow, ShirtColor::Blue, ShirtColor::Blue, ShirtColor::Yellow, ShirtColor::Yellow],
    };
    println!("curent store{:?} total:{}\n",store.shirt, store.shirt.len());
    // println!("curent store{:#?}",store.get_most_store());
    
    
    let usr_pref = Some(ShirtColor::Blue);
    let _give_shirt = match store.getaway(usr_pref) {
        Ok(shirt) =>{
            println!("give 2:{:?}",shirt);
            println!("curent store{:?} total:{}\n",store.shirt, store.shirt.len());
            shirt
        },
        Err(err) =>{
            eprintln!("Err {err}");
            ShirtColor::NoneColor
        }
    };
    
    let usr_pref = None;
    let _give_shirt = match store.getaway(usr_pref) {
        Ok(shirt) =>{
            println!("give 2:{:?}",shirt);
            println!("curent store{:?} total:{}\n",store.shirt, store.shirt.len());
            shirt
        },
        Err(err) =>{
            eprintln!("Err {err}");
            ShirtColor::NoneColor
        }
    };

    test_closure();
}
