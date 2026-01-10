import sys
sys.path.insert(0, '.')

import grpc
import time
from chaser.oxide.v1 import (
    browser_pb2,
    browser_pb2_grpc,
    page_pb2,
    page_pb2_grpc,
    profile_pb2,
    profile_pb2_grpc,
    common_pb2,
)

def main():
    channel = grpc.insecure_channel('localhost:50051')
    
    browser_stub = browser_pb2_grpc.BrowserServiceStub(channel)
    page_stub = page_pb2_grpc.PageServiceStub(channel)
    profile_stub = profile_pb2_grpc.ProfileServiceStub(channel)
    
    try:
        # 1. 创建 Windows 指纹
        print("1. 创建 Windows 指纹...")
        profile_resp = profile_stub.CreateProfile(profile_pb2.CreateProfileRequest(
            type=profile_pb2.PROFILE_TYPE_WINDOWS
        ))
        profile_id = profile_resp.profile.profile_id
        print(f"   Profile ID: {profile_id}")
        print(f"   Platform (指纹): {profile_resp.profile.fingerprint.navigator.platform}")
        
        # 2. 启动浏览器
        print("\n2. 启动浏览器...")
        launch_resp = browser_stub.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        browser_id = launch_resp.browser_info.browser_id
        print(f"   Browser ID: {browser_id}")
        
        # 3. 创建页面
        print("\n3. 创建页面...")
        page_resp = page_stub.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_resp.page_info.page_id
        print(f"   Page ID: {page_id}")
        
        # 4. 应用配置（在导航之前）
        print("\n4. 应用指纹配置...")
        apply_resp = profile_stub.ApplyProfile(profile_pb2.ApplyProfileRequest(
            page_id=page_id,
            profile_id=profile_id
        ))
        print(f"   应用特性: {apply_resp.result.applied_features}")
        
        # 5. 导航到测试页面
        print("\n5. 导航到 example.com...")
        nav_resp = page_stub.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))
        print(f"   导航成功: {nav_resp.result.url}")
        
        # 等待页面加载
        time.sleep(2)
        
        # 6. 检查 platform
        print("\n6. 检查 navigator.platform...")
        eval_resp = page_stub.Evaluate(page_pb2.EvaluateRequest(
            page_id=page_id,
            expression="navigator.platform"
        ))
        result = eval_resp.result
        if result.HasField('string_value'):
            platform = result.string_value
            print(f"   navigator.platform = {platform}")
            if platform == "Win32":
                print("   ✓ Platform 正确!")
            else:
                print(f"   ✗ Platform 错误，期望 Win32，得到 {platform}")
        else:
            print(f"   结果类型: {result.type}")
            print(f"   字符串值: {result.string_value}")
        
        # 检查 userAgent
        print("\n7. 检查 navigator.userAgent...")
        eval_resp = page_stub.Evaluate(page_pb2.EvaluateRequest(
            page_id=page_id,
            expression="navigator.userAgent"
        ))
        result = eval_resp.result
        if result.HasField('string_value'):
            ua = result.string_value
            print(f"   navigator.userAgent = {ua[:80]}...")
            if "Windows" in ua:
                print("   ✓ User-Agent 包含 Windows")
            else:
                print("   ✗ User-Agent 不包含 Windows")
        
        # 清理
        page_stub.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        browser_stub.Close(browser_pb2.CloseRequest(browser_id=browser_id))
        
        print("\n测试完成！")
        
    except grpc.RpcError as e:
        print(f"RPC 错误: {e.code()} - {e.details()}")
    finally:
        channel.close()

if __name__ == "__main__":
    main()
