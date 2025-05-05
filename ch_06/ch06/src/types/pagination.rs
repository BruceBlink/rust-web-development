use std::collections::HashMap;
use handle_errors::Error;


#[derive(Debug)]
pub struct Pagination {
    pub start: usize,
    pub end: usize,
}

pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
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