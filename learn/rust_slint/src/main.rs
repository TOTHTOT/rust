/*
 * @Author: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @Date: 2024-11-21 13:32:09
 * @LastEditors: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @LastEditTime: 2024-11-21 13:35:07
 * @FilePath: \rust_slint\src\main.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
fn main() {
    // 运行slintUI窗体
   MainWindow::new().unwrap().run().unwrap();
}
// slint宏，创建 UI
slint::slint!{
    export component MainWindow inherits Window {
        title: "Main Window";
        width: 600px;
        height: 500px;
        // 定义一个 Text 组件
        Text{
            text: "Hello, world";
            color:blue;
        }
    }
}   
