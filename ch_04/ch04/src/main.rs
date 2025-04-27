use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use warp::{
    Filter,
    filters::cors::CorsForbidden,
    http::Method,
    http::StatusCode,
    reject::Reject,
    Rejection,
    Reply,
};
use warp::path::param;

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../question.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }

    fn add_question(mut self, question: Question) -> Self {
        self.questions.insert(question.id.clone(), question);
        self
    }
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

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(std::io::Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}


#[derive(Debug)]
struct InvalidId;

impl Reject for InvalidId {}

async fn get_questions(params: HashMap<String, String>,store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    //println!("{:?}", params);
    /*match params.get("start") {
        Some(start) => println!("{}", start),
        None => println!("No start value"),
    }*/
    let mut start = 0;
    if let Some(n) = params.get("start") {
        start = n.parse::<usize>().expect("Could not parse start");
    }
    println!("{}", start);
    let res: Vec<Question> = store.questions.values().cloned().collect();
    Ok(warp::reply::json(&res))
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            },
            Error::MissingParameters => write!(f, "Missing parameter")
        }
    }
}

impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params.get("start").unwrap().parse::<usize>().map_err(Error::ParseError)?,
            end: params.get("end").unwrap().parse::<usize>().map_err(Error::ParseError)?,
        });
    }
    Err(Error::MissingParameters)
}



#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_questions.with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}