use std::io::{Error, ErrorKind};
use std::str::FromStr;

#[derive(Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug)]
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

fn main() {
    let question = Question::new(
        QuestionId::from_str("1").expect("No id provided"),
        "What is Rust?".to_string(),
        "Rust is a systems programming language.".to_string(),
        Some(vec!["rust".to_string(), "programming".to_string()]),
    );
    println!("{:?}", question);
}
