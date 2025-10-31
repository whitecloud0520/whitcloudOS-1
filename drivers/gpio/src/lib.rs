//! RK3588 GPIO 驱动
//! 
//! # 参考资料
//! - RK3588 Technical Reference Manual Part1 Chapter 20 - GPIO
//! - Linux Kernel: drivers/gpio/gpio-rockchip.c
//! 
//! # 硬件特性
//! - 5个GPIO Bank (GPIO0-GPIO4)
//! - 每个Bank有32个引脚，分为4组 (A/B/C/D)
//! - 支持输入/输出模式
//! - 支持中断功能（本版本未实现）
//! 
//! # 使用示例
//! ```no_run
//! use gpio::{GpioBank, GpioPin, GpioDirection, GpioLevel};
//! 
//! let led = GpioPin::new(GpioBank::Gpio0, 13);
//! led.set_direction(GpioDirection::Output);
//! led.set_level(GpioLevel::High);
//! ```

#![no_std]

use core::ptr::{read_volatile, write_volatile};

/// RK3588 GPIO 寄存器基址
/// 
/// 这些地址来自 RK3588 TRM Table 20-1
pub const GPIO0_BASE: usize = 0xFD8A0000;
pub const GPIO1_BASE: usize = 0xFEC20000;
pub const GPIO2_BASE: usize = 0xFEC30000;
pub const GPIO3_BASE: usize = 0xFEC40000;
pub const GPIO4_BASE: usize = 0xFEC50000;

/// GPIO 寄存器偏移
/// 
/// 参考: RK3588 TRM Section 20.2 - Register Description
const GPIO_SWPORT_DR: usize = 0x0000;      // 数据寄存器 (读写引脚电平)
const GPIO_SWPORT_DDR: usize = 0x0004;     // 方向寄存器 (0=输入, 1=输出)
const GPIO_EXT_PORT: usize = 0x0050;       // 外部端口寄存器 (只读, 读取实际引脚电平)

/// GPIO Bank 枚举
/// 
/// RK3588 有 5 个 GPIO Bank
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioBank {
    /// GPIO Bank 0 (系统相关 IO)
    Gpio0 = 0,
    /// GPIO Bank 1 (通用 IO)
    Gpio1 = 1,
    /// GPIO Bank 2 (通用 IO)
    Gpio2 = 2,
    /// GPIO Bank 3 (通用 IO)
    Gpio3 = 3,
    /// GPIO Bank 4 (通用 IO)
    Gpio4 = 4,
}

/// GPIO 引脚方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioDirection {
    /// 输入模式
    Input = 0,
    /// 输出模式
    Output = 1,
}

/// GPIO 电平
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioLevel {
    /// 低电平 (0V)
    Low = 0,
    /// 高电平 (3.3V)
    High = 1,
}

/// GPIO 引脚结构体
/// 
/// # 字段
/// - `base`: GPIO Bank 的寄存器基地址
/// - `pin`: 引脚号 (0-31)
/// 
/// # 引脚命名规则
/// GPIO 引脚通常命名为 `GPIOx_Yn`，其中：
/// - `x`: Bank 号 (0-4)
/// - `Y`: 组别 (A/B/C/D)
/// - `n`: 组内引脚号 (0-7)
/// 
/// 转换为引脚号的公式：
/// ```
/// pin = Group_Offset + n
/// Group_Offset: A=0, B=8, C=16, D=24
/// ```
/// 
/// 例如：GPIO0_B5 = Bank0, Group B, Pin 5 = 8 + 5 = Pin 13
pub struct GpioPin {
    base: usize,
    pin: u8,
}

impl GpioPin {
    /// 创建新的 GPIO 引脚实例
    /// 
    /// # 参数
    /// - `bank`: GPIO Bank (Gpio0-Gpio4)
    /// - `pin`: 引脚号 (0-31)
    /// 
    /// # Panic
    /// 如果 `pin` >= 32 则会 panic
    /// 
    /// # 示例
    /// ```no_run
    /// use gpio::{GpioBank, GpioPin};
    /// 
    /// // 创建 GPIO0_B5 (Pin 13)
    /// let led = GpioPin::new(GpioBank::Gpio0, 13);
    /// ```
    pub fn new(bank: GpioBank, pin: u8) -> Self {
        assert!(pin < 32, "Pin number must be less than 32");
        
        let base = match bank {
            GpioBank::Gpio0 => GPIO0_BASE,
            GpioBank::Gpio1 => GPIO1_BASE,
            GpioBank::Gpio2 => GPIO2_BASE,
            GpioBank::Gpio3 => GPIO3_BASE,
            GpioBank::Gpio4 => GPIO4_BASE,
        };
        
        Self { base, pin }
    }
    
    /// 设置引脚方向 (输入/输出)
    /// 
    /// # 参数
    /// - `direction`: 引脚方向
    /// 
    /// # 硬件操作
    /// 修改 GPIO_SWPORT_DDR 寄存器对应位
    /// - 0: 输入模式
    /// - 1: 输出模式
    pub fn set_direction(&self, direction: GpioDirection) {
        let addr = (self.base + GPIO_SWPORT_DDR) as *mut u32;
        unsafe {
            let mut val = read_volatile(addr);
            match direction {
                GpioDirection::Output => val |= 1 << self.pin,
                GpioDirection::Input => val &= !(1 << self.pin),
            }
            write_volatile(addr, val);
        }
    }
    
    /// 设置输出电平 (仅输出模式有效)
    /// 
    /// # 参数
    /// - `level`: 电平 (High/Low)
    /// 
    /// # 注意
    /// 调用此函数前应先调用 `set_direction(GpioDirection::Output)`
    /// 
    /// # 硬件操作
    /// 修改 GPIO_SWPORT_DR 寄存器对应位
    pub fn set_level(&self, level: GpioLevel) {
        let addr = (self.base + GPIO_SWPORT_DR) as *mut u32;
        unsafe {
            let mut val = read_volatile(addr);
            match level {
                GpioLevel::High => val |= 1 << self.pin,
                GpioLevel::Low => val &= !(1 << self.pin),
            }
            write_volatile(addr, val);
        }
    }
    
    /// 读取引脚电平
    /// 
    /// # 返回值
    /// 当前引脚的电平状态
    /// 
    /// # 硬件操作
    /// 读取 GPIO_EXT_PORT 寄存器对应位
    /// 
    /// # 注意
    /// - 输入模式：读取外部引脚实际电平
    /// - 输出模式：读取当前输出的电平
    pub fn get_level(&self) -> GpioLevel {
        let addr = (self.base + GPIO_EXT_PORT) as *const u32;
        unsafe {
            let val = read_volatile(addr);
            if (val & (1 << self.pin)) != 0 {
                GpioLevel::High
            } else {
                GpioLevel::Low
            }
        }
    }
    
    /// 翻转输出电平 (仅输出模式有效)
    /// 
    /// # 硬件操作
    /// 对 GPIO_SWPORT_DR 寄存器对应位执行 XOR 操作
    /// 
    /// # 用途
    /// 常用于 LED 闪烁等场景
    pub fn toggle(&self) {
        let addr = (self.base + GPIO_SWPORT_DR) as *mut u32;
        unsafe {
            let mut val = read_volatile(addr);
            val ^= 1 << self.pin;
            write_volatile(addr, val);
        }
    }
}

/// 引脚名称辅助函数
/// 
/// 将 GPIOx_Yn 格式转换为 (Bank, Pin) 元组
/// 
/// # 参数
/// - `bank`: Bank 号 (0-4)
/// - `group`: 组别 ('A', 'B', 'C', 'D')
/// - `pin`: 组内引脚号 (0-7)
/// 
/// # 返回值
/// (GpioBank, pin_number)
/// 
/// # 示例
/// ```no_run
/// use gpio::{parse_gpio_name, GpioPin};
/// 
/// // GPIO0_B5
/// let (bank, pin) = parse_gpio_name(0, 'B', 5);
/// let gpio = GpioPin::new(bank, pin);
/// ```
pub fn parse_gpio_name(bank: u8, group: char, pin: u8) -> (GpioBank, u8) {
    assert!(bank < 5, "Bank must be 0-4");
    assert!(pin < 8, "Pin must be 0-7");
    
    let bank_enum = match bank {
        0 => GpioBank::Gpio0,
        1 => GpioBank::Gpio1,
        2 => GpioBank::Gpio2,
        3 => GpioBank::Gpio3,
        4 => GpioBank::Gpio4,
        _ => unreachable!(),
    };
    
    let group_offset = match group.to_ascii_uppercase() {
        'A' => 0,
        'B' => 8,
        'C' => 16,
        'D' => 24,
        _ => panic!("Invalid group, must be A/B/C/D"),
    };
    
    (bank_enum, group_offset + pin)
}