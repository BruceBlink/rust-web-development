
use std::fmt::{Display, Formatter};

use warp::{Rejection, Reply};
use warp::body::BodyDeserializeError;
use warp::cors::CorsForbidden;
use warp::http::StatusCode;
use warp::reject::Reject;

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    InvalidRange, // 可以添加一个错误类型表示 start >= end
    QuestionNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            },
            Error::MissingParameters => write!(f, "Missing 'start' or 'end' parameter"), // 消息更清晰
            Error::InvalidRange => write!(f, "'start' must be less than 'end'"),
            Error::QuestionNotFound => write!(f, "question not found"),
        }
    }
}
impl Reject for Error {}

pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        // 对客户端参数错误使用 BAD_REQUEST (400)
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::BAD_REQUEST // <--- 修改点
        ))
    } else if let Some(cors_error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            cors_error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some (error) = r.find::<BodyDeserializeError>(){
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if r.is_not_found() { // 使用 is_not_found() 更明确
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else {
        // 处理其他未预期的 rejection
        eprintln!("Unhandled rejection: {:?}", r); // 最好记录下未处理的错误
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
