use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Answer, AnswerId, Question, QuestionId};

#[derive(Clone)]
pub struct Store {
    pub questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    pub answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        // 确保 questions.json 文件在编译时相对于 src/main.rs (或其他源文件) 的路径是正确的
        // 例如，如果 main.rs 在 src/ 下，questions.json 应该在项目根目录
        let file = include_str!("../question.json"); // 假设文件在项目根目录
        serde_json::from_str(file).expect("can't read questions.json")
    }
}