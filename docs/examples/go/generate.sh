#!/bin/bash
# Chaser-Oxide gRPC Go 客户端代码生成脚本
#
# 使用方法:
#   1. 确保已安装 protoc 编译器
#   2. 安装 Go protobuf 插件:
#      go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
#      go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
#   3. 在项目根目录运行: bash docs/examples/go/generate.sh
#
# 生成的代码将输出到 protos/ 目录

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}开始生成 Go gRPC 客户端代码...${NC}"

# 获取项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROTO_DIR="${PROJECT_ROOT}/protos"
OUTPUT_DIR="${PROJECT_ROOT}/protos"

# 检查 protoc 是否安装
if ! command -v protoc &> /dev/null; then
    echo -e "${YELLOW}错误: 未找到 protoc 编译器${NC}"
    echo "请安装 protoc: https://grpc.io/docs/protoc-installation/"
    exit 1
fi

# 检查 Go 插件是否安装
if ! command -v protoc-gen-go &> /dev/null; then
    echo -e "${YELLOW}警告: 未找到 protoc-gen-go 插件${NC}"
    echo "请安装: go install google.golang.org/protobuf/cmd/protoc-gen-go@latest"
fi

if ! command -v protoc-gen-go-grpc &> /dev/null; then
    echo -e "${YELLOW}警告: 未找到 protoc-gen-go-grpc 插件${NC}"
    echo "请安装: go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest"
fi

# 创建输出目录
mkdir -p "${OUTPUT_DIR}"

# 生成 gRPC 代码
echo -e "${YELLOW}正在编译 proto 文件...${NC}"

protoc \
    --proto_path="${PROTO_DIR}" \
    --go_out="${OUTPUT_DIR}" \
    --go-grpc_out="${OUTPUT_DIR}" \
    "${PROTO_DIR}"/*.proto

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ 成功生成 Go gRPC 客户端代码${NC}"
    echo -e "${GREEN}  输出目录: ${OUTPUT_DIR}${NC}"
    echo ""
    echo "下一步:"
    echo "  1. 根据您的项目结构调整 docs/examples/go/*.go 中的导入路径"
    echo "  2. 运行示例: cd docs/examples/go && go run basic_client.go"
else
    echo -e "${YELLOW}✗ 代码生成失败${NC}"
    exit 1
fi
