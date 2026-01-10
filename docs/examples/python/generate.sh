#!/bin/bash
# Chaser-Oxide gRPC Python 客户端代码生成脚本
#
# 使用方法:
#   1. 确保已安装 Python 和 grpcio-tools
#      pip install grpcio grpcio-tools
#   2. 在项目根目录运行: bash docs/examples/python/generate.sh
#
# 生成的代码将输出到 chaser/oxide/v1/ 目录结构中

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}开始生成 Python gRPC 客户端代码...${NC}"

# 获取项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROTO_DIR="${PROJECT_ROOT}/protos"

# 检查 Python 是否安装
if ! command -v python3 &> /dev/null; then
    echo -e "${YELLOW}错误: 未找到 Python3${NC}"
    echo "请安装 Python 3.7 或更高版本"
    exit 1
fi

# 检查 grpcio-tools 是否安装
if ! python3 -c "import grpc_tools.protoc" &> /dev/null; then
    echo -e "${YELLOW}错误: 未找到 grpcio-tools${NC}"
    echo "请安装: pip install grpcio grpcio-tools"
    exit 1
fi

# 生成 gRPC 代码
echo -e "${YELLOW}正在编译 proto 文件...${NC}"

python3 -m grpc_tools.protoc \
    --proto_path="${PROTO_DIR}" \
    --python_out="${PROJECT_ROOT}" \
    --grpc_python_out="${PROJECT_ROOT}" \
    "${PROTO_DIR}/common.proto" \
    "${PROTO_DIR}/browser.proto" \
    "${PROTO_DIR}/page.proto" \
    "${PROTO_DIR}/element.proto" \
    "${PROTO_DIR}/profile.proto" \
    "${PROTO_DIR}/event.proto"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ 成功生成 Python gRPC 客户端代码${NC}"
    echo -e "${GREEN}  输出目录: ${PROJECT_ROOT}/chaser/oxide/v1/${NC}"
    echo ""
    echo "生成的目录结构:"
    echo "  chaser/"
    echo "  └── oxide/"
    echo "      └── v1/"
    echo "          ├── common_pb2.py"
    echo "          ├── browser_pb2.py"
    echo "          ├── browser_pb2_grpc.py"
    echo "          ├── page_pb2.py"
    echo "          ├── page_pb2_grpc.py"
    echo "          ├── element_pb2.py"
    echo "          ├── element_pb2_grpc.py"
    echo "          ├── profile_pb2.py"
    echo "          ├── profile_pb2_grpc.py"
    echo "          ├── event_pb2.py"
    echo "          └── event_pb2_grpc.py"
    echo ""
    echo "下一步:"
    echo "  1. 确保 chaser/oxide/v1/ 目录在您的 Python 路径中"
    echo "  2. 运行示例: cd docs/examples/python && python3 basic_client.py"
else
    echo -e "${YELLOW}✗ 代码生成失败${NC}"
    exit 1
fi
