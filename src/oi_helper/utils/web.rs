use std::io::Read;

use html_parser::{Dom, Node};


pub fn get_remotely(url: &str) -> anyhow::Result<String> {
    let mut res = reqwest::blocking::get(url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;
    if crate::is_debug() {
        println!("Got content from {}", url);
        println!("Status: {}", res.status());
        println!("Headers: \n{:#?}", res.headers());
        println!("Body: \n{}", body);
    }
    Ok(body)
}

pub fn get_luogu_problem_content(problem_id: &str) -> anyhow::Result<Dom> {
    let content = get_remotely(&format!("https://www.luogu.com.cn/problem/{}", problem_id))?;
    let dom_tree = Dom::parse(&content)?;
    Ok(dom_tree)
}

#[allow(unused_doc_comments)]
/// Search from the Node n.
fn search_from(n: &Node) -> Option<Vec<(String, String)>> {

    // Store the test cases.
    let mut test_cases = Vec::<(String, String)>::new();

    // Check if the n is an element,
    match n.element() {

        Some(el) => {

            // Started to parse.
            let mut buffer = (String::new(), String::new());

            /**
             * 0 means normal.
             * 1 means reading the test-in.
             * 2 means reading the test-out.
             */
            let mut next_type = 0;

            for i in &el.children {

                match next_type {
                    // Check each element

                    // Normal mode
                    0 => {
                        match i.element() {
                            Some(el) => {
                                // Check if it's before a input test-case.
                                if el.name == "h3" && el.children.len() != 0 && el.children[0].text().is_some() && el.children[0].text().unwrap().starts_with("输入样例") {
                                    next_type = 1;
                                    continue;
                                }
                                // Check if it's before a output test-case.
                                if el.name == "h3" && el.children.len() != 0 && el.children[0].text().is_some() && el.children[0].text().unwrap().starts_with("输出样例") {
                                    next_type = 2;
                                    continue;
                                }

                                // Otherwise, search its children, and merge the result.
                                if let Some(cases) = search_from(i) {
                                    for i in cases {
                                        test_cases.push(i);
                                    }
                                }
                            }
                            None => {}
                        }
                    }

                    // Reading in-case
                    1 => {
                        match i.element() {
                            // Checking the relationships between them.
                            // There SHOULD be no problem.
                            Some(el) => {
                                if el.name == "pre" && el.children.len() != 0 {
                                    if let Some(c) = el.children[0].element() {
                                        if c.name == "code" {
                                            if let Some(s) = el.children[0].element().unwrap().children[0].text() {
                                                buffer.0 = String::from(s.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&"));
                                            }
                                        }
                                    }
                                }
                                next_type = 0;
                            }
                            None => {
                                next_type = 0;
                            }
                        }
                    }

                    // Read the out-case
                    2 => {
                        // The same as 1.
                        match i.element() {
                            Some(el) => {
                                if el.name == "pre" && el.children.len() != 0 {
                                    if let Some(c) = el.children[0].element() {
                                        if c.name == "code" {
                                            if let Some(s) = el.children[0].element().unwrap().children[0].text() {
                                                // Update it.
                                                buffer.1 = String::from(s.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&"));
                                                test_cases.push(buffer);
                                                buffer = (String::new(), String::new());
                                            }
                                        }
                                    }
                                }
                                next_type = 0;
                            }
                            None => {
                                next_type = 0;
                            }
                        }
                    }

                    _ => {}
                }

            }
        }

        None => {
            return None;
        }

    }

    Some(test_cases)
}

pub fn get_test_case_from_luogu_tree(dom: &Dom) -> Vec::<(String, String)> {
    let mut test_cases = Vec::<(String, String)>::new();
    for i in &dom.children {
        if let Some(cases) = search_from(i) {
            for i in cases {
                test_cases.push(i);
            }
        }
    }
    test_cases
}
