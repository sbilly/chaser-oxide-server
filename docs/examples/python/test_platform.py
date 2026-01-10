import grpc
from chaser.oxide.v1 import common_pb2, profile_pb2, browser_pb2, page_pb2, profile_pb2_grpc

channel = grpc.insecure_channel('localhost:50051')
profile_stub = profile_pb2_grpc.ProfileServiceStub(channel)
browser_stub = browser_pb2_grpc.BrowserServiceStub(channel)
page_stub = page_pb2_grpc.PageServiceStub(channel)

# 创建浏览器和页面
browser_resp = browser_stub.CreateBrowser(browser_pb2.CreateBrowserRequest())
browser_id = browser_resp.browser_id
page_resp = page_stub.CreatePage(page_pb2.CreatePageRequest(browser_id=browser_id))
page_id = page_resp.page_info.page_id

# 创建配置
profile_resp = profile_stub.CreateProfile(profile_pb2.CreateProfileRequest(
    profile=profile_pb2.FingerprintProfile(
        name="Windows Test",
        fingerprint=profile_pb2.Fingerprint(
            headers=profile_pb2.HeadersFingerprint(user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
            navigator=profile_pb2.NavigatorFingerprint(
                platform="Win32",
                vendor="",
                hardware_concurrency=8,
                device_memory=8,
                language="en-US"
            )
        )
    )
))
profile_id = profile_resp.profile_id

# 应用配置
apply_resp = profile_stub.ApplyProfile(profile_pb2.ApplyProfileRequest(
    page_id=page_id,
    profile_id=profile_id
))
print(f"应用结果: {apply_resp.result.applied_features}")

# 立即检查
check = page_stub.EvaluateScript(page_pb2.EvaluateScriptRequest(
    page_id=page_id,
    script="navigator.platform"
))
print(f"当前平台 (应用后): {check.result.value}")

# 导航到example.com
page_stub.Navigate(page_pb2.NavigateRequest(page_id=page_id, url="https://example.com"))

# 检查平台
check2 = page_stub.EvaluateScript(page_pb2.EvaluateScriptRequest(
    page_id=page_id,
    script="navigator.platform"
))
print(f"平台 (导航后): {check2.result.value}")
