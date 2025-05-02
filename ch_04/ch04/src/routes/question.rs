use std::cmp;
use std::collections::HashMap;
use warp::{Rejection, Reply};
use warp::http::StatusCode;
use crate::{error, store};
use crate::store::Store;
use crate::types::pagination::extract_pagination;
use crate::types::question::{Question, QuestionId};

pub async fn get_questions(params: HashMap<String, String>,store: store::Store) -> Result<impl warp::Reply, warp::Rejection> {

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

pub async fn add_question(store: Store,
                      question: Question) -> Result<impl Reply, Rejection> {
    store.questions.write().await.insert(question.id.clone(), question);
    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::OK,
    ))
}

pub async fn update_question(id: String,
                         store: Store,
                         question: Question) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(error::Error::QuestionNotFound)),
    }
    Ok(warp::reply::with_status(
        "Question updated",
        StatusCode::OK,
    ))
}

pub async fn delete_question(id: String,
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
        None => Err(warp::reject::custom(error::Error::QuestionNotFound))
    }
}