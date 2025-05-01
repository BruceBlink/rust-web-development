use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::str::FromStr;
use std::cmp;
use std::sync::Arc; // 引入 cmp 用于 min/max

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use warp::{
    Filter,
    filters::cors::CorsForbidden,
    http::Method,
    http::StatusCode,
    reject::Reject, // 移除未使用的 InvalidId 后，这里可能不再需要显式引入 Reject，但保留也无妨
    Rejection,
    Reply,
    body::BodyDeserializeError,
};
// warp::path::param; // 这个 import 没有被使用，可以移除

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init()))
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        // 确保 questions.json 文件在编译时相对于 src/main.rs (或其他源文件) 的路径是正确的
        // 例如，如果 main.rs 在 src/ 下，questions.json 应该在项目根目录
        let file = include_str!("../question.json"); // 假设文件在项目根目录
        serde_json::from_str(file).expect("can't read questions.json")
    }

    // 这个方法在原始代码中没有被调用，如果需要添加问题的功能，可以取消注释
    // fn add_question(mut self, question: Question) -> Self {
    //     self.questions.insert(question.id.clone(), question);
    //     self
    // }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

// 这个 new 方法在原始代码中没有被调用，可以保留或移除
// impl Question {
//     fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
//         Question {
//             id,
//             title,
//             content,
//             tags,
//         }
//     }
// }

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.is_empty() {
            Err(std::io::Error::new(ErrorKind::InvalidInput, "No id provided"))
        } else {
            Ok(QuestionId(id.to_string()))
        }
    }
}

// 移除了未使用的 InvalidId struct 和 impl Reject for InvalidId

async fn get_questions(params: HashMap<String, String>,store: Store) -> Result<impl warp::Reply, warp::Rejection> {

    if params.is_empty() {
        // 没有查询参数，返回所有问题
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    } else {
        // 有查询参数，尝试提取分页信息
        match extract_pagination(params) {
            Ok(pagination) => {
                let all_questions: Vec<Question> = store.questions.read().await.values().cloned().collect();
                let total_len = all_questions.len();

                // 确保 start 和 end 不会越界
                let start = cmp::min(pagination.start, total_len);
                let end = cmp::min(pagination.end, total_len);

                // 确保 start <= end，如果 start > end，返回空结果
                if start >= end {
                    let empty_questions: Vec<Question> = Vec::new();
                    Ok(warp::reply::json(&empty_questions))
                } else {
                    // 安全地进行切片
                    let paginated_questions = &all_questions[start..end];
                    Ok(warp::reply::json(&paginated_questions))
                }
            },
            Err(e) => {
                // 如果提取分页参数失败，返回一个 Rejection
                Err(warp::reject::custom(e))
            }
        }
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
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

#[derive(Debug)]
enum Error {
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

// 让自定义 Error 可以被 warp 作为 rejection 处理
impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    // 同时获取 start 和 end 参数
    let start_str = params.get("start").ok_or(Error::MissingParameters)?;
    let end_str = params.get("end").ok_or(Error::MissingParameters)?;

    // 解析参数
    let start = start_str.parse::<usize>().map_err(Error::ParseError)?;
    let end = end_str.parse::<usize>().map_err(Error::ParseError)?;

    // （可选）可以在这里就检查 start < end
    // if start >= end {
    //     return Err(Error::InvalidRange);
    // }

    Ok(Pagination { start, end })
}

async fn add_question(store: Store,
                      question: Question) -> Result<impl Reply, Rejection> {
    store.questions.write().await.insert(question.id.clone(), question);
    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::OK,
    ))
}

async fn update_question(id: String,
                         store: Store,   
                         question: Question) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }
    Ok(warp::reply::with_status(
        "Question updated",
        StatusCode::OK,
    ))
}

async fn delete_question(id: String,
                         store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => {
           return Ok(
                warp::reply::with_status(
                    "Question deleted",
                    StatusCode::OK,
                )
            )
        },
        None => Err(warp::reject::custom(Error::QuestionNotFound))
    }
}

#[tokio::main]
async fn main() {
    // 准备一个示例 questions.json 文件在项目根目录
    // 例如：
    // {
    //   "q1": { "id": "q1", "title": "First Question", "content": "Content of Q1", "tags": ["rust"] },
    //   "q2": { "id": "q2", "title": "Second Question", "content": "Content of Q2", "tags": ["web"] },
    //   "q3": { "id": "q3", "title": "Third Question", "content": "Content of Q3", "tags": ["warp"] }
    // }
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]); // GET 通常也需要允许

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query()) // 提取查询参数 HashMap<String, String>
        .and(store_filter.clone()) // 注入 store
        .and_then(get_questions); // 调用处理函数

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);
    // 注意：recover 需要放在应用 CORS *之前* 或 *之后*，取决于你想如何处理 CORS 错误
    // 通常放在应用 CORS 之后，这样 CORS 错误（如 CorsForbidden）也能被 return_error 捕获
    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .recover(return_error) // 捕获 get_questions 内部或 filter 链产生的 Rejection
        .with(cors); // 应用 CORS 策略

    println!("Server starting on http://127.0.0.1:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}