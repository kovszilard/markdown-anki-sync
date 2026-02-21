use notes_to_anki::anki::Response;
use notes_to_anki::anki_sync::DocumentSyncPlan;
use notes_to_anki::parser::document::parse_document;
use std::env;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <markdown-file>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let contents = match std::fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let (rest, doc) = match parse_document(&contents) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    };

    if !rest.is_empty() {
        eprintln!("Warning: unparsed remaining input ({} bytes)", rest.len());
    }

    let sync_plan = DocumentSyncPlan::from_document(doc);

    let (synced_document, created_count, updated_count) =
        sync_plan.sync(|request| {
            ureq::post("http://localhost:8765")
                .send_json(request)
                .and_then(|mut body| body.body_mut().read_json::<Response>())
                .ok()
        })?;

    println!("Created: {}, Updated: {}", created_count, updated_count);

    std::fs::write(filename, synced_document.raw())?;
    Ok(())
}
