use handle_errors::return_error;
use warp::{
    Filter,
    http::Method, // 移除未使用的 InvalidId 后，这里可能不再需要显式引入 Reject，但保留也无妨,
};

use crate::routes::answer::add_answer;
use crate::routes::question::{add_question, delete_question, get_questions, update_question};
use crate::store::Store;
use crate::types::answer::{Answer, AnswerId};
use crate::types::question::{Question, QuestionId};

mod routes;
mod types;
mod store;


#[tokio::main]
async fn main() {
    // 准备一个示例 questions.json 文件在项目根目录
    // 例如：
    // {
    //   "q1": { "id": "q1", "title": "First Question", "content": "Content of Q1", "tags": ["rust"] },
    //   "q2": { "id": "q2", "title": "Second Question", "content": "Content of Q2", "tags": ["web"] },
    //   "q3": { "id": "q3", "title": "Third Question", "content": "Content of Q3", "tags": ["warp"] }
    // }
    // 初始化日志记录器
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::error!("This is an error!");
    log::info!("This is info!");
    log::warn!("This is a warning!");
    let log = warp::log::custom(|info| {
       log::info!("{} {} {} {:?} from {} with {:?}",
                 info.method(),
                 info.path(),
                 info.status(),
                 info.elapsed(),
                 info.remote_addr().unwrap(),
                 info.request_headers()
       );
    });
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let id_filter = warp::any().map(|| uuid::Uuid::new_v4().to_string());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]); // GET 通常也需要允许

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query()) // 提取查询参数 HashMap<String, String>
        .and(store_filter.clone()) // 注入 store
        .and(id_filter)
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

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    // 注意：recover 需要放在应用 CORS *之前* 或 *之后*，取决于你想如何处理 CORS 错误
    // 通常放在应用 CORS 之后，这样 CORS 错误（如 CorsForbidden）也能被 return_error 捕获
    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .recover(return_error) // 捕获 get_questions 内部或 filter 链产生的 Rejection
        .with(cors) // 应用 CORS 策略
        .with(log);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}