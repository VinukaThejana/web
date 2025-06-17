use pulldown_cmark::{Options, Parser, html};
use scraper::{Html, Selector};

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

    let document = Html::parse_document(&html_output);
    let table_selector = Selector::parse("table").unwrap();
    let tables: Vec<_> = document.select(&table_selector).collect();
    if tables.is_empty() {
        return html_output;
    }

    let mut modified_html = html_output;

    for table_element in tables {
        let th_selector = Selector::parse("thead > tr > th").unwrap();
        let headers: Vec<String> = table_element
            .select(&th_selector)
            .map(|th| th.text().collect::<String>())
            .collect();
        if headers.is_empty() {
            continue;
        }

        let mut new_table_str = String::new();

        new_table_str.push_str("<table");
        for (name, value) in table_element.value().attrs() {
            new_table_str.push_str(&format!(" {}=\"{}\"", name, value.replace('"', "&quot;")));
        }
        new_table_str.push('>');

        // Re-add the thead as it is
        let thead_selector = Selector::parse("thead").unwrap();
        if let Some(thead) = table_element.select(&thead_selector).next() {
            new_table_str.push_str(&thead.html());
        }

        let tbody_selector = Selector::parse("tbody").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        if let Some(tbody) = table_element.select(&tbody_selector).next() {
            new_table_str.push_str("<tbody>");
            for tr in tbody.select(&tr_selector) {
                new_table_str.push_str("<tr");
                for (name, value) in tr.value().attrs() {
                    new_table_str.push_str(&format!(
                        " {}=\"{}\"",
                        name,
                        value.replace('"', "&quot;")
                    ));
                }
                new_table_str.push('>');

                for (i, td) in tr.select(&td_selector).enumerate() {
                    let header_text = headers.get(i).map_or("", |h| h.as_str());

                    new_table_str.push_str("<td data-label=\"");
                    new_table_str.push_str(&header_text.replace('"', "&quot;"));
                    new_table_str.push('"');
                    // Avoid duplicating the data-label if somehow it is already added
                    for (name, value) in td.value().attrs() {
                        if name.to_lowercase() != "data-label" {
                            new_table_str.push_str(&format!(
                                " {}=\"{}\"",
                                name,
                                value.replace('"', "&quot;")
                            ));
                        }
                    }
                    new_table_str.push('>');
                    new_table_str.push_str(&td.inner_html());
                    new_table_str.push_str("</td>");
                }

                new_table_str.push_str("</tr>");
            }
            new_table_str.push_str("</tbody>");
        }

        new_table_str.push_str("</table>");

        let original_table_html = table_element.html();
        modified_html = modified_html.replace(&original_table_html, &new_table_str);
    }

    modified_html
}
