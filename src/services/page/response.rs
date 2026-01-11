//! 响应构建辅助函数
//!
//! 此模块提供统一的响应构建函数，消除 RPC 方法中的重复代码。

#![allow(clippy::result_large_err)]

use tonic::{Response, Status};
use crate::Error;
use crate::chaser_oxide::v1::{
    ErrorCode, Error as ProtoError,
    Empty,
};

/// 将内部错误转换为 proto 错误
pub fn error_to_proto(error: Error) -> ProtoError {
    let code = match &error {
        Error::PageNotFound(_) => ErrorCode::PageClosed,
        Error::Timeout(_) => ErrorCode::Timeout,
        Error::NavigationFailed(_) => ErrorCode::NavigationFailed,
        Error::ScriptExecutionFailed(_) => ErrorCode::EvaluationFailed,
        Error::Configuration(_) => ErrorCode::InvalidArgument,
        _ => ErrorCode::Internal,
    };

    ProtoError {
        code: code.into(),
        message: error.to_string(),
        details: std::collections::HashMap::new(),
    }
}

/// 将内部错误转换为 tonic Status
pub fn error_to_status(error: Error) -> Status {
    let proto_error = error_to_proto(error);
    let status_code = match proto_error.code() {
        ErrorCode::PageClosed => tonic::Code::NotFound,
        ErrorCode::Timeout => tonic::Code::DeadlineExceeded,
        ErrorCode::NavigationFailed => tonic::Code::Aborted,
        ErrorCode::EvaluationFailed => tonic::Code::Internal,
        ErrorCode::InvalidArgument => tonic::Code::InvalidArgument,
        _ => tonic::Code::Internal,
    };

    Status::new(status_code, proto_error.message)
}

/// 成功响应构建器
pub struct SuccessResponse;

impl SuccessResponse {
    /// 创建简单的成功响应（仅包含 Empty）
    pub fn empty<T>() -> Result<Response<T>, Status>
    where
        T: From<Empty>,
    {
        // 这个函数需要根据具体的响应类型来实现
        // 由于 tonic 的响应类型各不相同，这里只是一个占位符
        // 实际使用时需要在每个 handler 中具体实现
        Err(Status::unimplemented("SuccessResponse::empty needs specific implementation"))
    }

    /// 创建带数据的成功响应
    pub fn with_data<T, D>(data: D) -> Result<Response<T>, Status>
    where
        T: From<D>,
    {
        Ok(Response::new(data.into()))
    }
}

/// 错误响应构建器
pub struct ErrorResponse;

impl ErrorResponse {
    /// 从内部错误创建响应
    pub fn from_error<T>(error: Error) -> Result<Response<T>, Status>
    where
        T: From<ProtoError>,
    {
        let proto_error = error_to_proto(error);
        Ok(Response::new(proto_error.into()))
    }

    /// 从 tonic Status 创建响应
    pub fn from_status<T>(status: Status) -> Result<Response<T>, Status> {
        Err(status)
    }

    /// 创建无效参数错误响应
    pub fn invalid_argument<T>(message: &str) -> Result<Response<T>, Status> {
        Err(Status::invalid_argument(message))
    }

    /// 创建未实现错误响应
    pub fn unimplemented<T>(message: &str) -> Result<Response<T>, Status> {
        Err(Status::unimplemented(message))
    }
}

/// 页面操作结果
///
/// 用于统一处理页面操作的返回结果
pub enum PageOperationResult<T> {
    /// 操作成功
    Success(T),
    /// 页面未找到
    PageNotFound(String),
    /// 操作失败
    Failed(Error),
}

impl<T> PageOperationResult<T> {
    /// 将结果转换为 tonic 响应
    pub fn to_response<R, F>(self, success_mapper: F) -> Result<Response<R>, Status>
    where
        R: From<ProtoError>,
        F: FnOnce(T) -> R,
    {
        match self {
            PageOperationResult::Success(data) => {
                Ok(Response::new(success_mapper(data)))
            }
            PageOperationResult::PageNotFound(page_id) => {
                let error = Error::page_not_found(&page_id);
                ErrorResponse::from_error(error)
            }
            PageOperationResult::Failed(error) => {
                ErrorResponse::from_error(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_proto_page_not_found() {
        let error = Error::page_not_found("test-page");
        let proto_error = error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::PageClosed as i32);
    }

    #[test]
    fn test_error_to_proto_timeout() {
        let error = Error::timeout("Operation timed out");
        let proto_error = error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::Timeout as i32);
    }

    #[test]
    fn test_error_to_proto_navigation_failed() {
        let error = Error::navigation_failed("Navigation failed");
        let proto_error = error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::NavigationFailed as i32);
    }

    #[test]
    fn test_error_to_proto_script_execution_failed() {
        let error = Error::script_execution_failed("Script error");
        let proto_error = error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::EvaluationFailed as i32);
    }

    #[test]
    fn test_error_to_proto_internal() {
        let error = Error::internal("Internal error");
        let proto_error = error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::Internal as i32);
    }
}
