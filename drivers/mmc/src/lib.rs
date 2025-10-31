//! RK3588 SDMMC 驱动
//! 参考: Linux kernel drivers/mmc/host/dw_mmc-rockchip.c
//! 芯片手册: RK3588 TRM Part1 Chapter 16 - SDMMC

#![no_std]

use core::ptr::{read_volatile, write_volatile};

/// SDMMC0 基址 (TF卡接口)
pub const SDMMC0_BASE: usize = 0xFE2C0000;

/// SDMMC 寄存器偏移
const SDMMC_CTRL: usize = 0x000;      // 控制寄存器
const SDMMC_PWREN: usize = 0x004;     // 电源使能寄存器
const SDMMC_CLKDIV: usize = 0x008;    // 时钟分频寄存器
const SDMMC_CLKENA: usize = 0x010;    // 时钟使能寄存器
const SDMMC_TMOUT: usize = 0x014;     // 超时寄存器
const SDMMC_CTYPE: usize = 0x018;     // 总线宽度寄存器
const SDMMC_BLKSIZ: usize = 0x01C;    // 块大小寄存器
const SDMMC_BYTCNT: usize = 0x020;    // 字节计数寄存器
const SDMMC_INTMASK: usize = 0x024;   // 中断屏蔽寄存器
const SDMMC_CMDARG: usize = 0x028;    // 命令参数寄存器
const SDMMC_CMD: usize = 0x02C;       // 命令寄存器
const SDMMC_RESP0: usize = 0x030;     // 响应寄存器0
const SDMMC_RESP1: usize = 0x034;     // 响应寄存器1
const SDMMC_RESP2: usize = 0x038;     // 响应寄存器2
const SDMMC_RESP3: usize = 0x03C;     // 响应寄存器3
const SDMMC_STATUS: usize = 0x048;    // 状态寄存器
const SDMMC_FIFOTH: usize = 0x04C;    // FIFO 阈值寄存器
const SDMMC_CDETECT: usize = 0x050;   // 卡检测寄存器

/// 控制寄存器位定义
const CTRL_RESET: u32 = 1 << 0;           // 控制器复位
const CTRL_FIFO_RESET: u32 = 1 << 1;      // FIFO 复位
const CTRL_DMA_RESET: u32 = 1 << 2;       // DMA 复位
const CTRL_INT_ENABLE: u32 = 1 << 4;      // 全局中断使能
const CTRL_DMA_ENABLE: u32 = 1 << 5;      // DMA 使能

/// 命令寄存器位定义
const CMD_START: u32 = 1 << 31;           // 开始命令
const CMD_WAIT_PRVDATA: u32 = 1 << 13;    // 等待前一个数据传输完成
const CMD_SEND_INIT: u32 = 1 << 15;       // 发送初始化序列

/// SD 卡命令定义
const CMD0_GO_IDLE_STATE: u32 = 0;
const CMD8_SEND_IF_COND: u32 = 8;
const CMD55_APP_CMD: u32 = 55;
const ACMD41_SD_SEND_OP_COND: u32 = 41;

#[derive(Debug)]
pub enum MmcError {
    InitFailed,
    ResetTimeout,
    CommandTimeout,
    CardNotPresent,
    UnsupportedCard,
}

pub struct SdMmc {
    base: usize,
}

impl SdMmc {
    /// 创建新的 SDMMC 实例
    pub fn new(base: usize) -> Self {
        Self { base }
    }
    
    /// 初始化 SDMMC 控制器
    pub fn init(&self) -> Result<(), MmcError> {
        // 1. 检测卡是否插入
        if !self.card_detect() {
            return Err(MmcError::CardNotPresent);
        }
        
        // 2. 复位控制器
        self.reset()?;
        
        // 3. 使能电源
        self.power_on();
        
        // 4. 设置时钟为 400kHz (识别模式)
        self.set_clock(400_000)?;
        
        // 5. 设置总线宽度为 1-bit
        self.set_bus_width(1);
        
        // 6. 设置超时
        self.set_timeout(0xFFFFFF);
        
        // 7. 配置 FIFO
        self.configure_fifo();
        
        Ok(())
    }
    
    /// 复位控制器
    fn reset(&self) -> Result<(), MmcError> {
        unsafe {
            let ctrl_addr = (self.base + SDMMC_CTRL) as *mut u32;
            
            // 发起复位
            write_volatile(
                ctrl_addr,
                CTRL_RESET | CTRL_FIFO_RESET | CTRL_DMA_RESET
            );
            
            // 等待复位完成
            let mut timeout = 10000;
            while read_volatile(ctrl_addr) & 0x07 != 0 {
                timeout -= 1;
                if timeout == 0 {
                    return Err(MmcError::ResetTimeout);
                }
            }
        }
        Ok(())
    }
    
    /// 使能电源
    fn power_on(&self) {
        unsafe {
            let pwren_addr = (self.base + SDMMC_PWREN) as *mut u32;
            write_volatile(pwren_addr, 1);
        }
    }
    
    /// 设置时钟频率
    fn set_clock(&self, freq: u32) -> Result<(), MmcError> {
        unsafe {
            let clkena_addr = (self.base + SDMMC_CLKENA) as *mut u32;
            let clkdiv_addr = (self.base + SDMMC_CLKDIV) as *mut u32;
            
            // 1. 禁用时钟
            write_volatile(clkena_addr, 0);
            self.update_clock();
            
            // 2. 设置分频系数
            // 假设源时钟为 50MHz
            let src_clk = 50_000_000;
            let div = if freq > 0 {
                (src_clk / (2 * freq)) & 0xFF
            } else {
                0
            };
            write_volatile(clkdiv_addr, div);
            
            // 3. 使能时钟
            write_volatile(clkena_addr, 1);
            self.update_clock();
        }
        Ok(())
    }
    
    /// 更新时钟配置
    fn update_clock(&self) {
        unsafe {
            let cmd_addr = (self.base + SDMMC_CMD) as *mut u32;
            write_volatile(cmd_addr, CMD_START | CMD_WAIT_PRVDATA | (1 << 21));
            
            // 等待命令完成
            let mut timeout = 10000;
            while read_volatile(cmd_addr) & CMD_START != 0 {
                timeout -= 1;
                if timeout == 0 {
                    break;
                }
            }
        }
    }
    
    /// 设置总线宽度
    fn set_bus_width(&self, width: u32) {
        unsafe {
            let ctype_addr = (self.base + SDMMC_CTYPE) as *mut u32;
            let val = match width {
                1 => 0x0,       // 1-bit
                4 => 0x1,       // 4-bit
                8 => 0x10000,   // 8-bit
                _ => 0x0,
            };
            write_volatile(ctype_addr, val);
        }
    }
    
    /// 设置超时值
    fn set_timeout(&self, timeout: u32) {
        unsafe {
            let tmout_addr = (self.base + SDMMC_TMOUT) as *mut u32;
            write_volatile(tmout_addr, timeout);
        }
    }
    
    /// 配置 FIFO
    fn configure_fifo(&self) {
        unsafe {
            let fifoth_addr = (self.base + SDMMC_FIFOTH) as *mut u32;
            // RX threshold = 7, TX threshold = 8, DMA burst size = 4
            let fifoth = (7 << 16) | (8 << 0) | (2 << 28);
            write_volatile(fifoth_addr, fifoth);
        }
    }
    
    /// 检测卡是否插入
    pub fn card_detect(&self) -> bool {
        unsafe {
            let cdetect_addr = (self.base + SDMMC_CDETECT) as *const u32;
            // 卡检测引脚低电平表示卡已插入
            read_volatile(cdetect_addr) & 0x1 == 0
        }
    }
    
    /// 发送命令
    pub fn send_command(&self, cmd: u32, arg: u32) -> Result<u32, MmcError> {
        unsafe {
            // 1. 设置命令参数
            let cmdarg_addr = (self.base + SDMMC_CMDARG) as *mut u32;
            write_volatile(cmdarg_addr, arg);
            
            // 2. 发送命令
            let cmd_addr = (self.base + SDMMC_CMD) as *mut u32;
            write_volatile(cmd_addr, CMD_START | cmd);
            
            // 3. 等待命令完成
            let mut timeout = 10000;
            while read_volatile(cmd_addr) & CMD_START != 0 {
                timeout -= 1;
                if timeout == 0 {
                    return Err(MmcError::CommandTimeout);
                }
            }
            
            // 4. 读取响应
            let resp0_addr = (self.base + SDMMC_RESP0) as *const u32;
            Ok(read_volatile(resp0_addr))
        }
    }
    
    /// 读取块数据
    pub fn read_block(&self, block_addr: u32, buffer: &mut [u8]) -> Result<(), MmcError> {
        // TODO: 实现块读取功能
        // 这需要实现完整的 SD 卡协议
        Ok(())
    }
    
    /// 写入块数据
    pub fn write_block(&self, block_addr: u32, buffer: &[u8]) -> Result<(), MmcError> {
        // TODO: 实现块写入功能
        Ok(())
    }
}