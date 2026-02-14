#!/bin/bash
# 项目编译测试脚本

set -e

echo "=== net-rs 项目编译测试 ==="
echo ""

# 检查 Rust 工具链
if ! command -v rustc &> /dev/null; then
    echo "错误: 未找到 rustc，请先安装 Rust"
    echo "访问 https://rustup.rs/ 获取安装说明"
    exit 1
fi

echo "Rust 工具链版本:"
rustc --version
cargo --version
echo ""

# 进入项目目录
cd /home/iori/.openclaw/workspace/code/rust/net-rs

echo "=== 清理之前的构建 ==="
cargo clean || true

echo ""
echo "=== 检查代码 ==="
cargo check --all-targets --all-features 2>&1 | head -50

echo ""
echo "=== 构建项目 ==="
cargo build 2>&1 | tail -20

echo ""
echo "=== 构建完成 ==="
if [ -f target/debug/nt ]; then
    echo "✅ 可执行文件已生成: target/debug/nt"
    ls -lh target/debug/nt
else
    echo "❌ 可执行文件未找到"
fi