fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("args is empty");
        std::process::exit(1);
    }
    let url = args[0].clone();
    let output = args[1].clone();
    let body = reqwest::blocking::get(&url)?.text()?;
    let md = html2md::parse_html(&body);
    std::fs::write(&output, md)?;
    println!("Converted {}s to {}", url, output);
    Ok(())
}
