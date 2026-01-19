#!/usr/bin/env python3
"""Test script to verify killpg implementation for chromium process cleanup."""

import grpc
import time
import subprocess
import sys

# Import generated protobuf classes
sys.path.insert(0, '.')
from chaser.oxide.v1 import browser_pb2, browser_pb2_grpc, common_pb2

def run_test(host="192.168.31.150:50051"):
    """Test that all chromium processes are killed on close."""

    print("=" * 60)
    print("Testing killpg implementation for chromium cleanup")
    print("=" * 60)

    # Connect to server
    channel = grpc.insecure_channel(host)
    stub = browser_pb2_grpc.BrowserServiceStub(channel)

    try:
        # Step 1: Create browser
        print("\n[Step 1] Creating browser...")
        request = browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(
                headless=True
            )
        )
        response = stub.Launch(request)
        browser_id = response.browser_info.browser_id
        print(f"  ✓ Browser created: {browser_id}")

        # Step 2: Wait for chromium processes to start
        print("\n[Step 2] Waiting for chromium processes to start...")
        time.sleep(2)

        # Step 3: Check chromium processes
        print("\n[Step 3] Checking chromium processes...")
        result = subprocess.run(
            ["docker", "exec", "chaser-oxide-server", "ps", "aux"],
            capture_output=True,
            text=True
        )
        chromium_lines = [line for line in result.stdout.split('\n') if 'chromium' in line.lower()]

        if not chromium_lines:
            print("  ✗ ERROR: No chromium processes found!")
            return False

        print(f"  ✓ Found {len(chromium_lines)} chromium process(es):")
        for line in chromium_lines:
            parts = line.split()
            if len(parts) >= 2:
                pid = parts[1]
                cmd = ' '.join(parts[10:]) if len(parts) > 10 else ''
                print(f"    PID {pid}: {cmd[:60]}...")

        # Step 4: Close browser
        print("\n[Step 4] Closing browser...")
        stub.Close(browser_pb2.CloseRequest(browser_id=browser_id))
        print(f"  ✓ Browser close request sent")

        # Step 5: Wait for processes to terminate and zombie reaper to run
        print("\n[Step 5] Waiting for process termination and zombie reaping...")
        time.sleep(7)  # Wait for zombie reaper task (runs every 5s)

        # Step 6: Verify no chromium processes remain
        print("\n[Step 6] Verifying cleanup...")
        result = subprocess.run(
            ["docker", "exec", "chaser-oxide-server", "ps", "aux"],
            capture_output=True,
            text=True
        )
        remaining_lines = [line for line in result.stdout.split('\n') if 'chromium' in line.lower()]

        if remaining_lines:
            print(f"  ✗ FAIL: {len(remaining_lines)} chromium process(es) still running:")
            for line in remaining_lines:
                parts = line.split()
                if len(parts) >= 2:
                    pid = parts[1]
                    print(f"    PID {pid}: {line}")
            return False
        else:
            print("  ✓ SUCCESS: All chromium processes terminated!")
            return True

    except grpc.RpcError as e:
        print(f"  ✗ RPC Error: {e.code()}: {e.details()}")
        return False
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return False
    finally:
        channel.close()

if __name__ == "__main__":
    success = run_test()
    print("\n" + "=" * 60)
    if success:
        print("TEST PASSED: killpg implementation works correctly")
        print("=" * 60)
        sys.exit(0)
    else:
        print("TEST FAILED: chromium processes not properly cleaned up")
        print("=" * 60)
        sys.exit(1)
