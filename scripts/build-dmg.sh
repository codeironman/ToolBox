#!/usr/bin/env bash
# 一键打包 macOS .dmg 安装包
# 流程: cargo build --release -> cargo bundle -> hdiutil 打包 dmg
# 用法: ./scripts/build-dmg.sh
set -euo pipefail

# 切到项目根目录
cd "$(dirname "$0")/.."

# 检查 cargo-bundle 是否安装
if ! cargo bundle --version &>/dev/null; then
  echo "错误: 未安装 cargo-bundle，请先运行: cargo install cargo-bundle" >&2
  exit 1
fi

echo "==> 1/3 编译 release 二进制（LTO + 体积优化，稍等）..."
cargo build --release

echo "==> 2/3 生成 Toolbox.app..."
cargo bundle --release

APP="target/release/bundle/osx/Toolbox.app"
if [ ! -d "$APP" ]; then
  echo "错误: 未找到 $APP" >&2
  exit 1
fi

# 从 Cargo.toml 读取版本号，用于命名 dmg
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
DMG="target/release/Toolbox-${VERSION}.dmg"

echo "==> 3/3 打包 dmg..."
rm -f "$DMG"
hdiutil create -volname "Toolbox" -srcfolder "$APP" -ov -format UDZO "$DMG" >/dev/null

echo ""
echo "✅ 打包完成"
SIZE=$(ls -lh "$DMG" | awk '{print $5}')
echo "   版本: $VERSION"
echo "   大小: $SIZE"
echo "   路径: $(pwd)/$DMG"
