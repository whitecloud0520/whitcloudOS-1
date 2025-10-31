#!/bin/bash
# WhitcloudOS-1 构建脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT=$(cd "$(dirname "$0")/.." && pwd)
TARGET=aarch64-unknown-none
OUTPUT_DIR="$PROJECT_ROOT/output"

echo -e "${GREEN}==================================="
echo "  Building WhitcloudOS-1"
echo "===================================${NC}"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 检查工具链
echo -e "${YELLOW}[0/5] Checking toolchain...${NC}"
if ! rustc --version > /dev/null 2>&1; then
    echo -e "${RED}Error: Rust not installed!${NC}"
    exit 1
fi

if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo -e "${YELLOW}Installing $TARGET...${NC}"
    rustup target add $TARGET
fi

# 构建 GPIO 驱动
echo -e "${YELLOW}[1/5] Building GPIO driver...${NC}"
cd "$PROJECT_ROOT/drivers/gpio"
cargo build --release --target=$TARGET
echo -e "${GREEN}✓ GPIO driver built${NC}"

# 构建 UART 驱动
echo -e "${YELLOW}[2/5] Building UART driver...${NC}"
cd "$PROJECT_ROOT/drivers/uart"
cargo build --release --target=$TARGET
echo -e "${GREEN}✓ UART driver built${NC}"

# 构建 MMC 驱动
echo -e "${YELLOW}[3/5] Building MMC driver...${NC}"
cd "$PROJECT_ROOT/drivers/mmc"
cargo build --release --target=$TARGET
echo -e "${GREEN}✓ MMC driver built${NC}"

# 构建应用程序
echo -e "${YELLOW}[4/5] Building applications...${NC}"
cd "$PROJECT_ROOT/rust-app"
cargo build --release --target=$TARGET
echo -e "${GREEN}✓ Applications built${NC}"

# 复制输出文件
echo -e "${YELLOW}[5/5] Copying binaries...${NC}"
cp "$PROJECT_ROOT/target/$TARGET/release/led_blink" "$OUTPUT_DIR/" 2>/dev/null || true
cp "$PROJECT_ROOT/target/$TARGET/release/uart_hello" "$OUTPUT_DIR/" 2>/dev/null || true
cp "$PROJECT_ROOT/target/$TARGET/release/mmc_test" "$OUTPUT_DIR/" 2>/dev/null || true

# 显示文件信息
echo -e "${GREEN}==================================="
echo "  Build completed!"
echo "===================================${NC}"
echo -e "${YELLOW}Output directory: $OUTPUT_DIR${NC}"
ls -lh "$OUTPUT_DIR/"

echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "  1. Convert binary: objcopy -O binary output/uart_hello output/uart_hello.bin"
echo "  2. Flash to SD card: sudo dd if=output/uart_hello.bin of=/dev/sdX bs=4M"
echo "  3. Boot and connect serial console"