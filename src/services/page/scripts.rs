//! JavaScript 脚本常量
//!
//! 此模块集中管理所有在页面中执行的 JavaScript 脚本，
//! 提高代码可维护性和可测试性。

/// 页面快照构建脚本
///
/// 构建页面的可访问性树，包括节点信息、角色、属性等
pub const SNAPSHOT_SCRIPT: &str = r#"
(() => {
    function buildTree(node, depth = 0) {
        if (depth > 100) return null; // 防止无限递归

        const nodeInfo = {
            node_id: node.id || Math.random().toString(36).substr(2, 9),
            role: node.getAttribute?.('role') || node.tagName?.toLowerCase() || '',
            name: node.getAttribute?.('aria-label') || node.textContent?.substr(0, 100) || '',
            description: node.getAttribute?.('aria-describedby') || '',
            tag_name: node.tagName?.toLowerCase() || '',
            attributes: [],
            children: [],
            is_visible: node.offsetParent !== null,
            is_interactive: ['button', 'a', 'input', 'select', 'textarea'].includes(node.tagName?.toLowerCase() || '')
        };

        // 获取重要属性
        if (node.id) nodeInfo.attributes.push(`id=${node.id}`);
        if (node.className) nodeInfo.attributes.push(`class=${node.className}`);
        if (node.type) nodeInfo.attributes.push(`type=${node.type}`);

        // 处理子节点
        if (node.children) {
            for (let child of node.children) {
                const childInfo = buildTree(child, depth + 1);
                if (childInfo) {
                    nodeInfo.children.push(childInfo.node_id);
                }
            }
        }

        return nodeInfo;
    }

    return JSON.stringify(buildTree(document.body));
})()
"#;

/// 获取 Cookies 脚本
///
/// 从 document.cookie 解析并返回结构化的 Cookie 信息
pub const GET_COOKIES_SCRIPT: &str = r#"
(() => {
    return document.cookie.split(';').map(cookie => {
        const [name, value] = cookie.trim().split('=');
        return {
            name: name || '',
            value: value || '',
            domain: window.location.hostname,
            path: '/',
            expires: 0,
            size: cookie.length,
            http_only: false,
            secure: window.location.protocol === 'https:',
            session: true,
            same_site: 'Lax'
        };
    }).filter(c => c.name);
})()
"#;

/// 清除 Cookies 脚本
///
/// 将所有 Cookie 的过期时间设置为过去，从而删除它们
pub const CLEAR_COOKIES_SCRIPT: &str = r#"
(() => {
    const cookies = document.cookie.split(';');
    cookies.forEach(cookie => {
        const [name] = cookie.trim().split('=');
        if (name) {
            document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
        }
    });
    return true;
})()
"#;

/// 等待选择器出现脚本
///
/// 轮询检查指定的 CSS 选择器是否出现在页面中
///
/// # 参数
/// - `selector`: CSS 选择器字符串
pub const WAIT_FOR_SELECTOR_SCRIPT: &str = r#"
((selector) => {
    return new Promise((resolve) => {
        const check = () => {
            const element = document.querySelector(selector);
            if (element) {
                resolve(true);
            } else {
                setTimeout(check, 100);
            }
        };
        check();
    });
})
"#;

/// 等待导航到指定 URL 脚本
///
/// 轮询检查当前页面 URL 是否等于目标 URL
///
/// # 参数
/// - `url`: 目标 URL 字符串
pub const WAIT_FOR_URL_SCRIPT: &str = r#"
((url) => {
    return new Promise((resolve) => {
        const check = () => {
            if (window.location.href === url) {
                resolve(true);
            } else {
                setTimeout(check, 100);
            }
        };
        check();
    });
})
"#;

/// 获取性能指标脚本
///
/// 使用 Performance API 获取页面加载和渲染性能指标
pub const GET_METRICS_SCRIPT: &str = r#"
(() => {
    if (!window.performance || !window.performance.timing) {
        return null;
    }

    const timing = window.performance.timing;
    const navigationStart = timing.navigationStart;

    return {
        timestamp: Date.now().toString(),
        layout_duration: timing.domContentLoadedEventEnd - timing.domContentLoadedEventStart,
        recalculate_style_duration: timing.domComplete - timing.domLoading,
        documents: document.querySelectorAll('document').length || 1,
        frames: window.frames.length,
        js_event_listeners: window.performance.getEntriesByType('resource').length,
        layouts: [],
        style_recalcs: []
    };
})()
"#;

/// 覆盖权限脚本
///
/// 覆盖 navigator.permissions.query 方法，使所有权限都返回 "granted"
///
/// # 参数
/// - `permissions`: 权限列表（JSON 字符串）
pub const OVERRIDE_PERMISSIONS_SCRIPT: &str = r#"
((permissions) => {
    // 覆盖权限 API
    const originalQuery = navigator.permissions.query;
    navigator.permissions.query = (name) => {
        return Promise.resolve({ state: 'granted' });
    };
    return true;
})
"#;

/// 设置离线模式脚本
///
/// 覆盖 navigator.onLine 和 navigator.connection 属性
///
/// # 参数
/// - `offline`: 是否离线（布尔值）
pub const SET_OFFLINE_MODE_SCRIPT: &str = r#"
((offline) => {
    // 覆盖 navigator.onLine
    Object.defineProperty(navigator, 'onLine', {
        get: () => offline
    });

    // 覆盖 window.navigator.connection
    if (navigator.connection) {
        Object.defineProperty(navigator.connection, 'effectiveType', {
            get: () => 'slow-2g'
        });
    }

    return true;
})
"#;

/// 设置缓存启用脚本
///
/// 覆盖 fetch API 以控制缓存行为
///
/// # 参数
/// - `enabled`: 是否启用缓存（布尔值）
pub const SET_CACHE_ENABLED_SCRIPT: &str = r#"
((enabled) => {
    // 覆盖 fetch API 以控制缓存
    const originalFetch = window.fetch;
    window.fetch = function(...args) {
        const options = args[1] || {};
        if (!enabled) {
            options.cache = 'no-store';
        }
        return originalFetch.apply(this, [args[0], options]);
    };
    return true;
})
"#;

/// 设置地理位置脚本
///
/// 覆盖 navigator.geolocation API 以模拟地理位置
///
/// # 参数
/// - `latitude`: 纬度
/// - `longitude`: 经度
/// - `accuracy`: 精度
pub const SET_GEOLOCATION_SCRIPT: &str = r#"
((latitude, longitude, accuracy) => {
    // 覆盖 navigator.geolocation
    const mockGeolocation = {
        getCurrentPosition: (success) => {
            success({
                coords: {
                    latitude: latitude,
                    longitude: longitude,
                    accuracy: accuracy,
                    altitude: null,
                    altitudeAccuracy: null,
                    heading: null,
                    speed: null
                },
                timestamp: Date.now()
            });
        },
        watchPosition: (success) => {
            success({
                coords: {
                    latitude: latitude,
                    longitude: longitude,
                    accuracy: accuracy,
                    altitude: null,
                    altitudeAccuracy: null,
                    heading: null,
                    speed: null
                },
                timestamp: Date.now()
            });
            return 0;
        },
        clearWatch: () => {}
    };

    Object.defineProperty(navigator, 'geolocation', {
        get: () => mockGeolocation
    });

    return true;
})
"#;

/// 设置 User Agent 脚本
///
/// 覆盖 navigator.userAgent 属性
///
/// # 参数
/// - `userAgent`: User Agent 字符串
pub const SET_USER_AGENT_SCRIPT: &str = r#"
((userAgent) => {
    Object.defineProperty(navigator, 'userAgent', {
        get: () => userAgent
    });
    return true;
})
"#;

/// 获取页面标题脚本
pub const GET_TITLE_SCRIPT: &str = "document.title";

/// 获取页面 URL 脚本
pub const GET_URL_SCRIPT: &str = "window.location.href";

/// 窗口聚焦脚本
pub const WINDOW_FOCUS_SCRIPT: &str = "window.focus()";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_script_exists() {
        assert!(!SNAPSHOT_SCRIPT.is_empty());
        assert!(SNAPSHOT_SCRIPT.contains("buildTree"));
    }

    #[test]
    fn test_get_cookies_script_exists() {
        assert!(!GET_COOKIES_SCRIPT.is_empty());
        assert!(GET_COOKIES_SCRIPT.contains("document.cookie"));
    }

    #[test]
    fn test_wait_for_selector_script_exists() {
        assert!(!WAIT_FOR_SELECTOR_SCRIPT.is_empty());
        assert!(WAIT_FOR_SELECTOR_SCRIPT.contains("querySelector"));
    }

    #[test]
    fn test_wait_for_url_script_exists() {
        assert!(!WAIT_FOR_URL_SCRIPT.is_empty());
        assert!(WAIT_FOR_URL_SCRIPT.contains("location.href"));
    }

    #[test]
    fn test_get_metrics_script_exists() {
        assert!(!GET_METRICS_SCRIPT.is_empty());
        assert!(GET_METRICS_SCRIPT.contains("performance.timing"));
    }
}
