//! RK3588 UART 驱动
//! 
//! # 参考资料
//! - RK3588 Technical Reference Manual Part1 Chapter 19 - UART
//! - Linux Kernel: drivers/tty/serial/8250/8250_dw.c
//! - TI 16550 UART Datasheet
//! 
//! # 硬件特性
//! - 兼容 16550 UART 标准
//! - 支持波特率 110 - 4Mbps
//! - 64 字节 TX/RX FIFO
//! - 支持硬件流控 (RTS/CTS)
//! 
//! # 使用示例
//! ```no_run
//! use uart::{Uart, UART2_BASE};
//! use core::fmt::Write;
//! 
//! let mut uart = Uart::new(UART2_BASE);
//! uart.init(115200);
//! writeln!(uart, "Hello, World!").unwrap();
//! ```

#![no_std]

use core::fmt;
use core::ptr::{read_volatile, write_volatile};

/// UART 控制器基址
/// 
/// RK3588 有 10 个 UART 控制器 (UART0-UART9)
/// 这里列出常用的几个
pub const UART0_BASE: usize = 0xFD890000;  // BT/Debug
pub const UART1_BASE: usize = 0xFEB40000;  // 通用
pub const UART2_BASE: usize = 0xFEB50000;  // **调试串口 (推荐)**
pub const UART3_BASE: usize = 0xFEB60000;  // 通用
pub const UART4_BASE: usize = 0xFEB70000;  // 通用

/// UART 寄存器偏移
/// 
/// 参考: 16550 UART 标准寄存器布局
const UART_RBR: usize = 0x00;   // 接收缓冲寄存器 (只读, DLAB=0)
const UART_THR: usize = 0x00;   // 发送保持寄存器 (只写, DLAB=0)
const UART_DLL: usize = 0x00;   // 分频器低字节 (DLAB=1)
const UART_DLH: usize = 0x04;   // 分频器高字节 (DLAB=1)
const UART_IER: usize = 0x04;   // 中断使能寄存器 (DLAB=0)
const UART_IIR: usize = 0x08;   // 中断识别寄存器 (只读)
const UART_FCR: usize = 0x08;   // FIFO 控制寄存器 (只写)
const UART_LCR: usize = 0x0C;   // 线控制寄存器
const UART_MCR: usize = 0x10;   // Modem 控制寄存器
const UART_LSR: usize = 0x14;   // 线状态寄存器
const UART_MSR: usize = 0x18;   // Modem 状态寄存器
const UART_USR: usize = 0x7C;   // UART 状态寄存器 (Designware 扩展)

/// 线状态寄存器 (LSR) 位定义
const LSR_DR: u32 = 1 << 0;     // 数据就绪
const LSR_OE: u32 = 1 << 1;     // 溢出错误
const LSR_PE: u32 = 1 << 2;     // 奇偶校验错误
const LSR_FE: u32 = 1 << 3;     // 帧错误
const LSR_BI: u32 = 1 << 4;     // Break 中断
const LSR_THRE: u32 = 1 << 5;   // 发送保持寄存器空
const LSR_TEMT: u32 = 1 << 6;   // 发送器空
const LSR_ERR: u32 = 1 << 7;    // FIFO 错误

/// 线控制寄存器 (LCR) 位定义
const LCR_WLS_5: u32 = 0x00;    // 5 位数据位
const LCR_WLS_6: u32 = 0x01;    // 6 位数据位
const LCR_WLS_7: u32 = 0x02;    // 7 位数据位
const LCR_WLS_8: u32 = 0x03;    // 8 位数据位
const LCR_STB: u32 = 1 << 2;    // 停止位 (0=1位, 1=1.5/2位)
const LCR_PEN: u32 = 1 << 3;    // 奇偶校验使能
const LCR_EPS: u32 = 1 << 4;    // 偶校验选择
const LCR_DLAB: u32 = 1 << 7;   // 分频器锁存访问位

/// FIFO 控制寄存器 (FCR) 位定义
const FCR_FIFO_EN: u32 = 1 << 0;    // FIFO 使能
const FCR_RX_FIFO_RST: u32 = 1 << 1; // 复位 RX FIFO
const FCR_TX_FIFO_RST: u32 = 1 << 2; // 复位 TX FIFO

/// UART 控制器结构体
pub struct Uart {
    base: usize,
}

impl Uart {
    /// 创建新的 UART 实例
    /// 
    /// # 参数
    /// - `base`: UART 控制器基址
    /// 
    /// # 示例
    /// ```no_run
    /// use uart::{Uart, UART2_BASE};
    /// let uart = Uart::new(UART2_BASE);
    /// ```
    pub const fn new(base: usize) -> Self {
        Self { base }
    }
    
    /// 初始化 UART 控制器
    /// 
    /// # 参数
    /// - `baudrate`: 波特率 (例如 115200)
    /// 
    /// # 配置
    /// - 数据位: 8
    /// - 停止位: 1
    /// - 校验位: 无
    /// - 流控: 无
    /// 
    /// # 波特率计算
    /// ```
    /// divisor = clock / (16 * baudrate)
    /// ```
    /// 假设 UART 时钟 24MHz，波特率 115200:
    /// ```
    /// divisor = 24,000,000 / (16 * 115200) = 13 (0x0D)
    /// ```
    /// 
    /// # 示例
    /// ```no_run
    /// use uart::{Uart, UART2_BASE};
    /// let uart = Uart::new(UART2_BASE);
    /// uart.init(115200);  // 初始化为 115200 8N1
    /// ```
    pub fn init(&self, baudrate: u32) {
        unsafe {
            // 1. 禁用中断
            let ier_addr = (self.base + UART_IER) as *mut u32;
            write_volatile(ier_addr, 0);
            
            // 2. 设置 DLAB=1 以访问分频器
            let lcr_addr = (self.base + UART_LCR) as *mut u32;
            write_volatile(lcr_addr, LCR_DLAB);
            
            // 3. 计算并设置分频器
            // 假设 UART 时钟源为 24MHz
            let clock = 24_000_000;
            let divisor = clock / (16 * baudrate);
            
            let dll_addr = (self.base + UART_DLL) as *mut u32;
            let dlh_addr = (self.base + UART_DLH) as *mut u32;
            write_volatile(dll_addr, (divisor & 0xFF) as u32);
            write_volatile(dlh_addr, ((divisor >> 8) & 0xFF) as u32);
            
            // 4. 清除 DLAB, 设置 8N1 (8位数据, 无校验, 1位停止)
            write_volatile(lcr_addr, LCR_WLS_8);
            
            // 5. 使能并复位 FIFO
            let fcr_addr = (self.base + UART_FCR) as *mut u32;
            write_volatile(fcr_addr, FCR_FIFO_EN | FCR_RX_FIFO_RST | FCR_TX_FIFO_RST);
        }
    }
    
    /// 发送一个字节
    /// 
    /// # 参数
    /// - `byte`: 要发送的字节
    /// 
    /// # 阻塞
    /// 此函数会等待发送缓冲区空闲
    pub fn putc(&self, byte: u8) {
        unsafe {
            // 等待发送保持寄存器空 (LSR[5] = 1)
            let lsr_addr = (self.base + UART_LSR) as *const u32;
            while (read_volatile(lsr_addr) & LSR_THRE) == 0 {
                // 自旋等待
            }
            
            // 写入数据到发送保持寄存器
            let thr_addr = (self.base + UART_THR) as *mut u32;
            write_volatile(thr_addr, byte as u32);
        }
    }
    
    /// 接收一个字节 (非阻塞)
    /// 
    /// # 返回值
    /// - `Some(byte)`: 收到数据
    /// - `None`: 接收缓冲区为空
    pub fn getc(&self) -> Option<u8> {
        unsafe {
            let lsr_addr = (self.base + UART_LSR) as *const u32;
            
            // 检查数据就绪位 (LSR[0])
            if (read_volatile(lsr_addr) & LSR_DR) != 0 {
                let rbr_addr = (self.base + UART_RBR) as *const u32;
                Some(read_volatile(rbr_addr) as u8)
            } else {
                None
            }
        }
    }
    
    /// 发送字符串
    /// 
    /// # 参数
    /// - `s`: 要发送的字符串
    /// 
    /// # 注意
    /// 遇到 `\n` 会自动发送 `\r\n` (CRLF)
    pub fn puts(&self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.putc(b'\r');  // 先发送 CR
            }
            self.putc(byte);
        }
    }
    
    /// 检查发送器是否空闲
    /// 
    /// # 返回值
    /// - `true`: 发送器空闲
    /// - `false`: 仍在发送数据
    pub fn is_tx_idle(&self) -> bool {
        unsafe {
            let lsr_addr = (self.base + UART_LSR) as *const u32;
            (read_volatile(lsr_addr) & LSR_TEMT) != 0
        }
    }
}

/// 实现 fmt::Write trait，支持 write! 和 writeln! 宏
impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);
        Ok(())
    }
}

/// 全局控制台 UART 实例（可选）
/// 
/// 用于实现 print! 和 println! 宏
static mut CONSOLE: Option<Uart> = None;

/// 初始化全局控制台
/// 
/// # 参数
/// - `base`: UART 基址
/// - `baudrate`: 波特率
/// 
/// # 安全性
/// 此函数使用 unsafe，应在系统启动时调用一次
pub fn init_console(base: usize, baudrate: u32) {
    unsafe {
        let uart = Uart::new(base);
        uart.init(baudrate);
        CONSOLE = Some(uart);
    }
}

/// print! 宏实现
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        unsafe {
            if let Some(ref mut uart) = $crate::CONSOLE {
                let _ = write!(uart, $($arg)*);
            }
        }
    }};
}

/// println! 宏实现
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {{
        $crate::print!($($arg)*);
        $crate::print!("\n");
    }};
}