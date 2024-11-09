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