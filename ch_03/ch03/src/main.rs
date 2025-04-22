use std::io::{Error, ErrorKind};
use std::str::FromStr;

use serde::Serialize;
use warp::{Filter, Rejection, Reply};
use warp::http::Method;
use warp::hyper::StatusCode;
use warp::reject::Reject;

#[derive(Debug, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct QuestionId(String);

impl FromStr for QuestionId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            true => Err(
                Error::new(ErrorKind::InvalidInput, "No id provide"),
            ),
            false => Ok(QuestionId(id.to_string())),
        }
    }
}

impl Question {
    fn new(id: QuestionId,
           title: String,
           content: String,
           tags: Option<Vec<String>>,
    ) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}


async fn get_question() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        QuestionId::from_str("1").expect("No id provide"),
        "First Question".to_string(),
        "Content of question".to_string(),
        Some(vec!("faq".to_string())),
    );
    match question.id.0.parse::<i32>() {
        Err(_) => {
            Err(warp::reject::custom(InvalidId))
        },
        Ok(_) => {
            Ok(warp::reply::json(
                &question
            )
            )
        }
    }
}

#[derive(Debug)]
struct InvalidId;

impl Reject for InvalidId {}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(_invalid_id) = r.find::<InvalidId>() {
        Ok(warp::reply::with_status(
            "No valid ID presented",
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found",
            StatusCode::NOT_FOUND,
        ))
    }
}



#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(
            &[Method::PUT, Method::DELETE, Method::GET, Method::POST]
        );
    let get_items = warp::get()
        .and(warp::path("question"))
        .and(warp::path::end())
        .and_then(get_question)  // 注意这里传入的是一个函数名而不是一个函数调用，
        .recover(return_error);
    let routes = get_items.with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
