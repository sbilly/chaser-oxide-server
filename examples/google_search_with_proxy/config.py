"""
代理配置管理模块

支持从环境变量加载代理配置,并转换为 chaser-oxide 所需格式。
优先级: ALL_PROXY > SOCKS_PROXY > HTTPS_PROXY > HTTP_PROXY
"""

import os
from typing import Optional
from dataclasses import dataclass


@dataclass
class ProxyConfig:
    """代理配置类

    支持从环境变量读取以下代理配置:
    - HTTP_PROXY / http_proxy: HTTP 代理
    - HTTPS_PROXY / https_proxy: HTTPS 代理
    - SOCKS_PROXY / socks_proxy: SOCKS 代理
    - ALL_PROXY / all_proxy: 全局代理 (最高优先级)
    - NO_PROXY / no_proxy: 代理绕过列表
    """

    http_proxy: Optional[str] = None
    https_proxy: Optional[str] = None
    socks_proxy: Optional[str] = None
    no_proxy: Optional[str] = None

    @classmethod
    def from_env(cls) -> 'ProxyConfig':
        """从环境变量加载代理配置

        按照标准环境变量名称读取代理配置,支持大写和小写格式。

        Returns:
            ProxyConfig: 代理配置实例
        """
        return cls(
            http_proxy=os.getenv('HTTP_PROXY') or os.getenv('http_proxy'),
            https_proxy=os.getenv('HTTPS_PROXY') or os.getenv('https_proxy'),
            socks_proxy=os.getenv('SOCKS_PROXY') or os.getenv('socks_proxy'),
            no_proxy=os.getenv('NO_PROXY') or os.getenv('no_proxy'),
        )

    def to_chaser_proxy(self) -> str:
        """转换为 chaser-oxide 代理格式

        按照 chaser-oxide 的优先级规则选择代理:
        1. ALL_PROXY / all_proxy (全局代理,最高优先级)
        2. SOCKS_PROXY (SOCKS 代理)
        3. HTTPS_PROXY (HTTPS 代理)
        4. HTTP_PROXY (HTTP 代理)

        Returns:
            str: 代理服务器地址,格式如 "http://proxy.example.com:8080"
                 或 "socks5://proxy.example.com:1080"
        """
        # 检查全局代理
        all_proxy = os.getenv('ALL_PROXY') or os.getenv('all_proxy')
        if all_proxy:
            return all_proxy

        # 按优先级检查其他代理
        if self.socks_proxy:
            return self.socks_proxy
        if self.https_proxy:
            return self.https_proxy
        if self.http_proxy:
            return self.http_proxy

        return ""

    def to_chaser_bypass_list(self) -> str:
        """转换为代理绕过列表

        将 NO_PROXY 环境变量转换为 chaser-oxide 所需的绕过列表格式。
        支持逗号分隔的多个域名或 IP 地址。

        Returns:
            str: 逗号分隔的绕过列表,如 "localhost,127.0.0.1,.example.com"
                 空字符串表示不绕过任何地址
        """
        if self.no_proxy:
            return self.no_proxy
        return ""

    def is_active(self) -> bool:
        """检查是否配置了代理

        Returns:
            bool: 如果配置了任何代理则返回 True
        """
        return bool(self.to_chaser_proxy())

    def __str__(self) -> str:
        """字符串表示

        Returns:
            str: 代理配置的可读描述
        """
        proxy = self.to_chaser_proxy()
        if not proxy:
            return "无代理配置"

        bypass = self.to_chaser_bypass_list()
        bypass_str = f", 绕过: {bypass}" if bypass else ""
        return f"代理: {proxy}{bypass_str}"
