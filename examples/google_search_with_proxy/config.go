// Package main 提供代理配置管理功能
//
// 支持从环境变量加载代理配置,并转换为 chaser-oxide 所需格式。
// 优先级: ALL_PROXY > SOCKS_PROXY > HTTPS_PROXY > HTTP_PROXY
package main

import (
	"fmt"
	"os"
)

// ProxyConfig 代理配置结构体
//
// 支持从环境变量读取以下代理配置:
// - HTTP_PROXY / http_proxy: HTTP 代理
// - HTTPS_PROXY / https_proxy: HTTPS 代理
// - SOCKS_PROXY / socks_proxy: SOCKS 代理
// - ALL_PROXY / all_proxy: 全局代理 (最高优先级)
// - NO_PROXY / no_proxy: 代理绕过列表
type ProxyConfig struct {
	HTTPProxy  string
	HTTPSProxy string
	SOCKSProxy string
	NoProxy    string
}

// NewProxyConfigFromEnv 从环境变量加载代理配置
//
// 按照标准环境变量名称读取代理配置,支持大写和小写格式。
//
// Returns:
//   *ProxyConfig: 代理配置实例
func NewProxyConfigFromEnv() *ProxyConfig {
	return &ProxyConfig{
		HTTPProxy:  getEnv("HTTP_PROXY", "http_proxy"),
		HTTPSProxy: getEnv("HTTPS_PROXY", "https_proxy"),
		SOCKSProxy: getEnv("SOCKS_PROXY", "socks_proxy"),
		NoProxy:    getEnv("NO_PROXY", "no_proxy"),
	}
}

// getEnv 获取环境变量（支持大写和小写）
//
// 按顺序尝试多个环境变量名称,返回第一个非空值。
//
// Parameters:
//   keys: 环境变量名称列表,按优先级排序
//
// Returns:
//   string: 第一个非空的环境变量值,如果都不存在则返回空字符串
func getEnv(keys ...string) string {
	for _, key := range keys {
		if val := os.Getenv(key); val != "" {
			return val
		}
	}
	return ""
}

// ToChaserProxy 转换为 chaser-oxide 代理格式
//
// 按照 chaser-oxide 的优先级规则选择代理:
// 1. ALL_PROXY / all_proxy (全局代理,最高优先级)
// 2. SOCKS_PROXY (SOCKS 代理)
// 3. HTTPS_PROXY (HTTPS 代理)
// 4. HTTP_PROXY (HTTP 代理)
//
// Returns:
//   string: 代理服务器地址,格式如 "http://proxy.example.com:8080"
//           或 "socks5://proxy.example.com:1080"
func (c *ProxyConfig) ToChaserProxy() string {
	// 检查全局代理
	if allProxy := getEnv("ALL_PROXY", "all_proxy"); allProxy != "" {
		return allProxy
	}

	// 按优先级检查其他代理
	if c.SOCKSProxy != "" {
		return c.SOCKSProxy
	}
	if c.HTTPSProxy != "" {
		return c.HTTPSProxy
	}
	if c.HTTPProxy != "" {
		return c.HTTPProxy
	}

	return ""
}

// ToChaserBypassList 转换为代理绕过列表
//
// 将 NO_PROXY 环境变量转换为 chaser-oxide 所需的绕过列表格式。
// 支持逗号分隔的多个域名或 IP 地址。
//
// Returns:
//   string: 逗号分隔的绕过列表,如 "localhost,127.0.0.1,.example.com"
//           空字符串表示不绕过任何地址
func (c *ProxyConfig) ToChaserBypassList() string {
	return c.NoProxy
}

// IsActive 检查是否配置了代理
//
// Returns:
//   bool: 如果配置了任何代理则返回 true
func (c *ProxyConfig) IsActive() bool {
	return c.ToChaserProxy() != ""
}

// String 返回代理配置的可读描述
//
// Returns:
//   string: 代理配置的字符串表示
func (c *ProxyConfig) String() string {
	proxy := c.ToChaserProxy()
	if proxy == "" {
		return "无代理配置"
	}

	bypass := c.ToChaserBypassList()
	bypassStr := ""
	if bypass != "" {
		bypassStr = fmt.Sprintf(", 绕过: %s", bypass)
	}

	return fmt.Sprintf("代理: %s%s", proxy, bypassStr)
}
