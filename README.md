whitcloudOS 开发
# WhitcloudOS

李鸿宇的嵌入式操作系统开发项目 - 基于 RK3588 开发板的 Rust 全栈实现

## 项目简介

WhitcloudOS 是一个面向 RK3588 开发板的轻量级嵌入式操作系统比赛项目，采用 Rust 语言从底层驱动到应用层全栈实现，注重内存安全、性能优化和工程化实践。

## 硬件平台

- **开发板**: RK3588 (ARM Cortex-A76/A55)
- **存储**: TF卡
- **调试**: UART2 串口 (115200 8N1)
- **外设**: GPIO、UART、SDMMC
- **散热**: 主动散热风扇

## 项目特色

✨ **Rust 全栈开发** - 从驱动层到应用层完全使用 Rust 实现  
🔒 **内存安全** - 利用 Rust 的所有权系统保证系统安全性  
⚡ **高性能** - Zero-cost 抽象，裸机性能  
🛠️ **工程化** - 完整的构建系统、文档和测试  
📚 **详细文档** - 中文文档，从零开始的开发指南

## 快速开始

### 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加 ARM64 目标
rustup target add aarch64-unknown-none
rustup component add rust-src

# 安装构建工具 (Ubuntu/Debian)
sudo apt install gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu
```

### 构建项目

```bash
# 克隆仓库
git clone https://github.com/whitecloud0520/whitcloudOS-1.git
cd whitcloudOS-1

# 构建所有组件
chmod +x scripts/build.sh
./scripts/build.sh
```

### 烧录到 TF 卡

```bash
# 将二进制文件转换为纯二进制格式
aarch64-linux-gnu-objcopy -O binary \
    output/uart_hello \
    output/uart_hello.bin

# 烧录镜像（请根据实际设备修改 /dev/sdX）
sudo ./scripts/flash.sh output/uart_hello.bin /dev/sdX
```

### 连接串口

```bash
# 使用 minicom
sudo minicom -D /dev/ttyUSB0 -b 115200

# 或使用 screen
sudo screen /dev/ttyUSB0 115200
```

## 项目结构

```
whitcloudOS-1/
├── README.md           # 项目说明
├── Cargo.toml          # Rust 工作空间配置
├── .cargo/
│   └── config.toml     # Cargo 构建配置
├── link.ld             # 链接脚本
├── bootloader/         # U-Boot 相关（规划中）
├── kernel/             # 内核配置和补丁（规划中）
├── drivers/            # 驱动代码
│   ├── gpio/           # GPIO 驱动
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── uart/           # 串口驱动
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── mmc/            # TF卡驱动
│       ├── Cargo.toml
│       └── src/lib.rs
├── rust-app/           # Rust 应用层
│   ├── Cargo.toml
│   ├── src/
│   │   └── start.s     # 启动汇编代码
│   └── examples/
│       ├── led_blink.rs   # LED 闪烁示例
│       ├── uart_hello.rs  # 串口输出示例
│       └── mmc_test.rs    # TF卡测试示例
├── scripts/            # 构建和烧录脚本
│   ├── build.sh        # 构建脚本
│   └── flash.sh        # 烧录脚本
├── docs/               # 文档
│   ├── hardware-setup.md      # 硬件连接指南
│   ├── development-setup.md   # 开发环境配置
│   ├── driver-development.md  # 驱动开发文档
│   └── troubleshooting.md     # 故障排查
├── tests/              # 测试代码（规划中）
└── output/             # 编译输出目录
```

## 开发路线图

### ✅ 第一阶段：基础系统（当前阶段）
- [ ] 项目结构搭建
- [ ] GPIO 驱动实现
- [ ] UART 串口驱动
- [ ] LED 闪烁示例
- [ ] 串口输出测试
- [ ] 基础文档完善

### 📋 第二阶段：存储支持
- [ ] SDMMC 控制器驱动
- [ ] SD 卡初始化流程
- [ ] 块设备读写接口
- [ ] FAT32 文件系统支持
- [ ] 文件读写测试


## 示例程序

### LED 闪烁
```rust
use gpio::{GpioBank, GpioPin, GpioDirection, GpioLevel};

let led = GpioPin::new(GpioBank::Gpio0, 13);
led.set_direction(GpioDirection::Output);

loop {
    led.set_level(GpioLevel::High);  // LED 亮
    delay_ms(500);
    led.set_level(GpioLevel::Low);   // LED 灭
    delay_ms(500);
}
```

### 串口输出
```rust
use uart::Uart;
use core::fmt::Write;

let mut uart = Uart::new(UART2_BASE);
uart.init(115200);

writeln!(uart, "Hello from Rust!").unwrap();
```

## 文档

- [硬件连接指南](docs/hardware-setup.md) - 引脚定义、连接方法
- [开发环境配置](docs/development-setup.md) - 工具链安装、IDE 配置
- [驱动开发文档](docs/driver-development.md) - 寄存器说明、开发规范
- [故障排查](docs/troubleshooting.md) - 常见问题与解决方法

## 参考资料

本项目在开发过程中参考了以下技术文档和开源项目：

### 官方技术文档

#### Rockchip RK3588
- **RK3588 Technical Reference Manual (TRM)**
  - 来源：Rockchip 官方文档
  - 内容：芯片寄存器定义、外设控制器规格
  - GPIO: Part1 Chapter 20
  - UART: Part1 Chapter 19  
  - SDMMC: Part1 Chapter 16

- **RK3588 Datasheet**
  - 来源：Rockchip 官方文档
  - 内容：电气特性、引脚定义、时序图

#### 标准协议规范
- **16550 UART Specification**
  - 来源：Texas Instruments
  - 文档：[TL16C550C Datasheet](https://www.ti.com/lit/ds/symlink/tl16c550c.pdf)
  - 内容：UART 控制器寄存器定义和操作流程

- **SD Card Physical Layer Specification**
  - 来源：SD Association
  - 网站：https://www.sdcard.org/downloads/pls/
  - 内容：SD 卡命令集、初始化流程、数据传输协议

- **ARM Architecture Reference Manual (ARMv8-A)**
  - 来源：ARM Limited
  - 内容：ARM64 指令集、系统寄存器、异常处理

### Linux 内核驱动参考

本项目驱动实现参考了 Linux 内核的相关驱动代码：

```bash
# Linux Kernel 源码仓库
Repository: https://github.com/torvalds/linux
License: GPL-2.0
```

**参考的驱动文件**：

- **GPIO 驱动**
  - 文件：`drivers/gpio/gpio-rockchip.c`
  - 作用：RK3588 GPIO 控制器驱动实现
  - 参考内容：寄存器操作、中断处理逻辑

- **UART 驱动**
  - 文件：`drivers/tty/serial/8250/8250_dw.c`
  - 文件：`drivers/tty/serial/8250/8250_port.c`
  - 作用：Designware 8250 兼容 UART 驱动
  - 参考内容：波特率计算、FIFO 配置、中断处理

- **SDMMC 驱动**
  - 文件：`drivers/mmc/host/dw_mmc.c`
  - 文件：`drivers/mmc/host/dw_mmc-rockchip.c`
  - 作用：Designware MMC 控制器驱动
  - 参考内容：命令发送、数据传输、DMA 配置

### Rust 嵌入式开发资源

#### 官方教程和文档

- **The Embedded Rust Book**
  - 网址：https://rust-embedded.github.io/book/
  - 内容：Rust 嵌入式开发入门、no_std 编程、硬件抽象层

- **The Embedonomicon**
  - 网址：https://docs.rust-embedded.org/embedonomicon/
  - 内容：从零构建嵌入式程序、链接脚本、启动代码

- **Rust API Guidelines**
  - 网址：https://rust-lang.github.io/api-guidelines/
  - 内容：Rust API 设计规范、最佳实践

#### Rust 嵌入式 Crate

- **cortex-a**
  - 仓库：https://github.com/rust-embedded/cortex-a
  - 用途：ARMv8-A (Cortex-A) 处理器支持
  - 参考内容：寄存器定义、系统调用包装

- **embedded-hal**
  - 仓库：https://github.com/rust-embedded/embedded-hal
  - 用途：嵌入式硬件抽象层标准接口
  - 参考内容：GPIO、UART、SPI 等外设抽象

- **volatile-register**
  - 仓库：https://github.com/rust-embedded/volatile-register
  - 用途：安全的 MMIO 寄存器访问
  - 参考内容：类型安全的寄存器读写

### 开源项目参考

- **rust-raspberrypi-OS-tutorials**
  - 仓库：https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials
  - 作者：Andre Richter
  - 内容：ARM64 裸机 Rust 操作系统教程
  - 参考内容：启动代码、异常处理、设备驱动框架

- **Tock OS**
  - 仓库：https://github.com/tock/tock
  - 官网：https://www.tockos.org/
  - 内容：Rust 编写的嵌入式操作系统
  - 参考内容：驱动架构、安全模型、进程隔离

- **Redox OS**
  - 仓库：https://github.com/redox-os/redox
  - 官网：https://www.redox-os.org/
  - 内容：Rust 编写的类 Unix 操作系统
  - 参考内容：系统调用设计、驱动框架

### 其他技术资源

- **OSDev Wiki**
  - 网址：https://wiki.osdev.org/
  - 内容：操作系统开发百科全书
  - 参考内容：启动流程、内存管理、设备驱动

- **ARM Developer Documentation**
  - 网址：https://developer.arm.com/documentation/
  - 内容：ARM 处理器、总线协议、调试接口

- **Rockchip Linux Kernel**
  - 仓库：https://github.com/rockchip-linux/kernel
  - 内容：Rockchip 官方维护的 Linux 内核
  - 参考内容：设备树、板级支持包

## 致谢

感谢以上所有开源项目、技术文档和社区贡献者的无私分享，为本项目提供了宝贵的参考和灵感。

特别感谢：
- Rust 嵌入式工作组（Rust Embedded WG）
- Linux 内核社区
- ARM 和 Rockchip 的技术文档团队

## 许可证

本项目采用 MIT 许可证开源。

```
MIT License

Copyright (c) 2025 whitecloud0520

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 作者

**李鸿宇** (@whitecloud0520)

- GitHub: https://github.com/whitecloud0520
- 项目：WhitcloudOS-1 - RK3588 Rust 嵌入式操作系统

## 贡献指南

欢迎提交 Issue 和 Pull Request！

在提交 PR 之前，请确保：
- 代码遵循 Rust 编码规范（`cargo fmt`）
- 通过所有测试（`cargo test`）
- 添加必要的文档注释
- 更新相关文档

## 联系方式

如有问题或建议，请通过以下方式联系：
- 提交 GitHub Issue
- 发送邮件至项目维护者

---

**最后更新时间**: 2025-10-31

**项目状态**: 🚧 开发中
