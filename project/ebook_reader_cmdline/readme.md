<!--
 * @Author: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @Date: 2024-11-09 21:26:28
 * @LastEditors: TOTHTOT 37585883+TOTHTOT@users.noreply.github.com
 * @LastEditTime: 2024-11-09 21:26:34
 * @FilePath: \ebook_reader_cmdline\readme.md
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->

# txt 阅读器

## 项目说明

- 项目使用`rust`编写, 运行平台`Ubuntu`;

## 待解决问题

1. - [ ] `term_width` 如果设置的和终端宽度不匹配会出现下一行时多插入换行或者没有成功回到行头问题, 目前在`vscode`的终端设置为`74`时功能正常, 使用`mobaxterm`可能由于字体等宽或者终端自动换行的原因导致刷新当前行失效问题;
2. - [ ]  `check_config()`增加文件合法性判断;
3. - [ ]  不允许打开文件失效的文件;
4. - [ ]  推出读书流程刷新掉当前行内容;
5. - [ ] 增加老板键;
6. - [ ]  增加`上一行`功能;

## 其他

1. 交叉编译

```shell
# 在 .cargo/config.toml 添加如下内容, 如果没有这个文件就创建
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

sudo apt-get install gcc-arm-linux-gnueabihf # 安装交叉编译器, 如果有就不要安装

rustup target add armv7-unknown-linux-gnueabihf # 确认 cargo 和 Rust 工具链是否配置为使用交叉编译环境

cargo build --target=armv7-unknown-linux-gnueabihf --release # 编译v3s平台程序

```