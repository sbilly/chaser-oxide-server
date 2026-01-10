#!/usr/bin/env python3
import sys
sys.path.insert(0, '.')

from chaser.oxide.v1 import common_pb2, page_pb2

# 创建一个测试响应来查看结构
response = page_pb2.EvaluateResponse()

# 打印所有可用字段
print("EvaluateResponse fields:")
print([x for x in dir(response) if not x.startswith('_')])
print("\n" + "="*50 + "\n")

# 检查 result 字段
print("Checking result field...")
print(f"HasField('result'): {response.HasField('result') if response.HasField('result') else 'False (field not set)'}")

# 设置一个字符串结果来测试
response.result.string_value = "Example Domain"
print(f"After setting string_value: {response.result}")

# 打印 result 的字段
print("\nResult fields:")
print([x for x in dir(response.result) if not x.startswith('_')])
print(f"\nstring_value: '{response.result.string_value}'")
print(f"HasField('string_value'): {response.result.HasField('string_value')}")
