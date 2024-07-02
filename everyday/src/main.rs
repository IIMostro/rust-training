mod collections;
mod lifecycle;
mod closure;
mod logging;

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;
    use reqwest;
    use html2md;
    use super::*;

    #[test]
    fn first_rust_test() {
        let url = "https://www.baidu.com/";
        let output = "rust.md";

        println!("Fetching url: {}", url);
        let body = reqwest::blocking::get(url).unwrap().text().unwrap();

        println!("Converting html to markdown...");
        let md = html2md::parse_html(&body);

        fs::write(output, md.as_bytes()).unwrap();
        println!("Converted markdown has been saved in {}.", output);
    }

    #[test]
    fn test_apply() {
        let value = 10;
        // 使用lambda表达式表示函数
        let f:fn(i32) -> i32 = |x| x + 1;
        let result = apply(value, f);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_int_address(){
        let v = 4;
        println!("v: {}", &v);
    }

    #[test]
    fn test_move_data(){
    }

    #[test]
    fn test_inner_borrow_mut(){
        let data = RefCell::new(1);
        {
            // 获得 RefCell 内部数据的可变借用
            let mut v = data.borrow_mut();
            *v += 1;
        }
        println!("data: {:?}", data.borrow());
    }

    #[test]
    fn test_inner_borrow_mut_2(){
        let data = RefCell::new(1);
        {
            let mut v = data.borrow_mut();
            *v += 1;
        }

        println!("data: {:?}", data.borrow());
    }

}

fn main() {
    for arg in std::env::args() {
        println!("{}", arg);
    }
}

fn apply(value:i32, f: fn(i32) -> i32) -> i32 {
    f(value)
}

#[derive(Debug, Copy, Clone)]
struct TopicId(u64);

impl TopicId {
    fn new(id: u64) -> TopicId {
        TopicId(id)
    }
}