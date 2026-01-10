#!/usr/bin/env python3
import sys
sys.path.insert(0, '.')

import grpc
from chaser.oxide.v1 import common_pb2, page_pb2, page_pb2_grpc, browser_pb2, browser_pb2_grpc

# 连接到服务器
channel = grpc.insecure_channel("localhost:50051")
page_stub = page_pb2_grpc.PageServiceStub(channel)
browser_stub = browser_pb2_grpc.BrowserServiceStub(channel)

# 启动浏览器
launch_response = browser_stub.Launch(browser_pb2.LaunchRequest(
    options=common_pb2.BrowserOptions(headless=True)
))

if launch_response.HasField('error'):
    print(f"启动失败: {launch_response.error.message}")
    exit(1)

browser_id = launch_response.browser_info.browser_id
print(f"浏览器已启动: {browser_id}")

# 创建页面
create_response = page_stub.CreatePage(page_pb2.CreatePageRequest(
    browser_id=browser_id,
    url="about:blank"
))

if create_response.HasField('error'):
    print(f"创建页面失败: {create_response.error.message}")
    exit(1)

page_id = create_response.page_info.page_id
print(f"页面已创建: {page_id}")

# 导航到 example.com
print("\n导航到 example.com...")
navigate_response = page_stub.Navigate(page_pb2.NavigateRequest(
    page_id=page_id,
    url="https://example.com"
))

if navigate_response.HasField('error'):
    print(f"导航失败: {navigate_response.error.message}")
    exit(1)

print(f"导航成功")

# 等待一下让页面加载完成
import time
time.sleep(2)

# 获取页面标题
print("\n执行 JavaScript 获取标题...")
eval_response = page_stub.Evaluate(page_pb2.EvaluateRequest(
    page_id=page_id,
    expression='document.title'
))

print("\n=== 调试信息 ===")
print(f"eval_response 类型: {type(eval_response)}")
print(f"eval_response 字段: {[x for x in dir(eval_response) if not x.startswith('_')]}")
print(f"\nHasField('error'): {eval_response.HasField('error') if hasattr(eval_response, 'HasField') else 'N/A'}")
print(f"HasField('result'): {eval_response.HasField('result') if hasattr(eval_response, 'HasField') else 'N/A'}")

# 检查 result 字段
if hasattr(eval_response, 'result') and eval_response.HasField('result'):
    result = eval_response.result
    print(f"\nresult 类型: {type(result)}")
    print(f"result 字段: {[x for x in dir(result) if not x.startswith('_')]}")
    print(f"\nresult.WhichOneof('response'): {result.WhichOneof('response')}")
    print(f"result.type: '{result.type}'")

    # 检查各个可能字段
    if result.HasField('string_value'):
        print(f"string_value: '{result.string_value}'")
    if result.HasField('int_value'):
        print(f"int_value: {result.int_value}")
    if result.HasField('double_value'):
        print(f"double_value: {result.double_value}")
    if result.HasField('bool_value'):
        print(f"bool_value: {result.bool_value}")

elif hasattr(eval_response, 'error') and eval_response.HasField('error'):
    print(f"\n错误: {eval_response.error}")
else:
    print("\n没有 result 也没有 error 字段被设置")
    print(f"完整响应: {eval_response}")

# 清理
page_stub.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
browser_stub.Close(browser_pb2.CloseRequest(browser_id=browser_id))
channel.close()
