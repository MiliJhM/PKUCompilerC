# 2023秋 北京大学编译原理实践: SysY 编译器

该项目使用Rust语言实现一个简易的、未经优化的SysY - Koopa IR - RISC-V编译器。

## 使用方法

首先 clone 本仓库:

```sh
git clone https://github.com/pku-minic/sysy-cmake-template.git
```

进入仓库目录后执行:

```sh
cargo run -- < -mode inputdir > < -o outputdir >
```

Cargo 将自动构建并运行该项目.

程序支持三种运行模式：

* -koopa: 该模式下，程序将输入的SysY程序编译到Koopa IR，并输出文本形式的.Koopa文件。
* -riscv: 该模式下，程序将输入的SysY程序编译到RV32IM范围内的RISC-V汇编文件。
* -perf:  该模式下，程序将输入的SysY程序编译，并得到经过优化的RISC-V汇编文件，用于性能测试。

