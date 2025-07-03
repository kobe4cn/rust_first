pub trait Formater {
    fn format(&self, input: &mut String) -> bool;
}
struct MarkdownFormater;
impl Formater for MarkdownFormater {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("Markdown!");
        true
    }
}

struct HtmlFormater;
impl Formater for HtmlFormater {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("Html!");
        true
    }
}

struct JsonFormater;
impl Formater for JsonFormater {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("Json!");
        true
    }
}

pub fn format(input: &mut String, formater: Vec<&dyn Formater>) {
    for f in formater {
        f.format(input);
    }
}

fn main() {
    let mut input = String::from("Hello, world!");
    let html: &dyn Formater = &HtmlFormater;
    let json: &dyn Formater = &JsonFormater;
    let markdown: &dyn Formater = &MarkdownFormater;
    let formater = vec![markdown, html, json];
    format(&mut input, formater);
    println!("{}", input);
}
