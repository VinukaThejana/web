use lol_html::{HtmlRewriter, Settings, element, text};
use pulldown_cmark::{Options, Parser, html};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
struct TableState {
    headers: Vec<String>,
    current_column_index: usize,
}

pub fn cmark(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_WIKILINKS);

    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let state = Rc::new(RefCell::new(TableState::default()));
    let mut output = Vec::new();

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                // Open links in a new tab if they are external links
                element!("a[href]", |el| {
                    let href = match el.get_attribute("href") {
                        Some(href) => href,
                        None => return Ok(()),
                    };
                    if href.starts_with("http://")
                        || href.starts_with("https://")
                        || href.starts_with("//")
                    {
                        el.set_attribute("target", "_blank")?;
                        el.set_attribute("rel", "noopener noreferrer")?;
                    }
                    Ok(())
                }),
                // Add data attributes to tables
                // Reset table state when a new table starts
                element!("table", |_| {
                    let mut state = state.borrow_mut();
                    state.headers.clear();
                    state.current_column_index = 0;
                    Ok(())
                }),
                // Capture the text content of each header cell `<th>`.
                // We must handle cases where text comes in multiple chunks.
                element!("thead > tr > th", |_| {
                    state.borrow_mut().headers.push(String::new());
                    Ok(())
                }),
                text!("thead > tr > th", |chunk| {
                    if let Some(last_header) = state.borrow_mut().headers.last_mut() {
                        last_header.push_str(chunk.as_str());
                    }
                    Ok(())
                }),
                element!("tbody > tr", |_| {
                    state.borrow_mut().current_column_index = 0;
                    Ok(())
                }),
                element!("tbody > tr > td", |el| {
                    let mut state = state.borrow_mut();
                    let header_txt = state
                        .headers
                        .get(state.current_column_index)
                        .cloned()
                        .unwrap_or_default();
                    el.set_attribute("data-label", &header_txt)?;
                    state.current_column_index += 1;
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(html_output.as_bytes()).unwrap();
    rewriter.end().unwrap();

    String::from_utf8(output).unwrap()
}
