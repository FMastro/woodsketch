//! `woodsketch-cli mcp`: WoodSketch as a Model Context Protocol server on stdin/stdout, so Claude
//! and other assistants can design with it. One drawing is kept between calls: build it from a
//! recipe (or open a file), check it (summary, cut list, wood movement, a picture), and save it
//! or write the shop sheets. Messages are JSON-RPC 2.0, one per line.

use std::io::{BufRead, Write};
use std::path::Path;

use serde_json::{Value, json};
use woodsketch_doc::{Document, cutlist, recipe};

/// The protocol version answered when a client asks for one this doesn't know.
const PROTOCOL: &str = "2025-06-18";

pub fn serve() {
    let mut drawing: Option<Document> = None;
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Some(reply) = handle(&line, &mut drawing) else { continue };
        let _ = writeln!(stdout, "{reply}");
        let _ = stdout.flush();
    }
}

/// One message in, at most one answer out (notifications get none).
pub fn handle(line: &str, drawing: &mut Option<Document>) -> Option<Value> {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "Not JSON" } }));
    };
    let id = message.get("id").cloned()?;
    let method = message["method"].as_str().unwrap_or("");
    let params = &message["params"];
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": params["protocolVersion"].as_str().unwrap_or(PROTOCOL),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "woodsketch", "version": env!("CARGO_PKG_VERSION") },
            "instructions": "WoodSketch designs woodworking projects. Call `guide` first: it explains the project recipe (JSON) that `build` takes. Then build, check with `summary`, `movement` and `render`, fix and rebuild, and `save` or write `cut_sheet` / `build_sheets` for the shop."
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => Ok(call(params["name"].as_str().unwrap_or(""), &params["arguments"], drawing)),
        other => Err(json!({ "code": -32601, "message": format!("No method {other}") })),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({ "name": name, "description": description, "inputSchema": { "type": "object", "properties": properties, "required": required } })
}

fn tools() -> Vec<Value> {
    let path = json!({ "type": "string", "description": "A file path" });
    vec![
        tool("guide", "How to write a WoodSketch project recipe, with an example. Read this before build.", json!({}), &[]),
        tool(
            "build",
            "Build the drawing from a project recipe (see guide). Replaces the current drawing. Returns a summary (size, parts by material, joints, wood movement) or every problem found, each naming the recipe item to fix.",
            json!({ "recipe": { "type": "object", "description": "The project recipe (JSON object, or a string of JSON)" } }),
            &["recipe"],
        ),
        tool("open", "Open a .woodsketch drawing, a recipe .json, or a SketchUp 2021+ .skp as the current drawing.", json!({ "path": path }), &["path"]),
        tool("save", "Save the current drawing as a .woodsketch file (the WoodSketch app opens it).", json!({ "path": path }), &["path"]),
        tool("summary", "The current drawing's overall size, parts by material with sizes, joints and wood movement warnings.", json!({}), &[]),
        tool(
            "cut_list",
            "The current drawing's cut list: each material's parts (label, name, count, length x width x thickness), and hardware.",
            json!({ "format": { "type": "string", "enum": ["text", "csv"] } }),
            &[],
        ),
        tool("movement", "The wood movement check: how much each solid-wood part moves across its grain, and where parts fight it.", json!({}), &[]),
        tool(
            "render",
            "A picture of the current drawing (PNG), zoomed to fit, to check how it looks.",
            json!({ "view": { "type": "string", "enum": ["iso", "top", "front", "right"] }, "width": { "type": "integer" }, "height": { "type": "integer" } }),
            &[],
        ),
        tool("cut_sheet", "Write the printable cut sheet (parts, what to buy, cutting diagrams) as an HTML file.", json!({ "path": path }), &["path"]),
        tool("build_sheets", "Write the build sheets (a page per part with its steps, then assembly) as an HTML file.", json!({ "path": path }), &["path"]),
        tool("export", "Export the current drawing as .obj (with .mtl) or .stl for other 3D programs.", json!({ "path": path, "millimeters": { "type": "boolean" } }), &["path"]),
    ]
}

/// Text content, marked as an error or not.
fn text(text: impl Into<String>, error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text.into() }], "isError": error })
}

fn call(name: &str, args: &Value, drawing: &mut Option<Document>) -> Value {
    let path = || args["path"].as_str().map(Path::new).ok_or("Give a \"path\".".to_owned());
    let current = |drawing: &Option<Document>| -> Result<(), String> {
        if drawing.is_none() { Err("No drawing yet: build one from a recipe, or open a file.".into()) } else { Ok(()) }
    };
    let result: Result<Value, String> = (|| match name {
        "guide" => Ok(text(crate::GUIDE, false)),
        "build" => {
            let json = match &args["recipe"] {
                Value::String(s) => s.clone(),
                Value::Null => return Err("Give a \"recipe\".".into()),
                other => other.to_string(),
            };
            let doc = crate::build(&json)?;
            let summary = recipe::summary(&doc);
            *drawing = Some(doc);
            Ok(text(format!("Built.\n{summary}"), false))
        }
        "open" => {
            let doc = crate::load(path()?)?;
            let summary = recipe::summary(&doc);
            *drawing = Some(doc);
            Ok(text(format!("Opened.\n{summary}"), false))
        }
        "save" => {
            current(drawing)?;
            let to = path()?;
            crate::save(drawing.as_ref().expect("checked"), to)?;
            Ok(text(format!("Saved {}.", to.display()), false))
        }
        "summary" => {
            current(drawing)?;
            Ok(text(recipe::summary(drawing.as_ref().expect("checked")), false))
        }
        "cut_list" => {
            current(drawing)?;
            let list = cutlist::cut_list(drawing.as_ref().expect("checked").scene());
            Ok(text(if args["format"].as_str() == Some("csv") { cutlist::to_csv(&list) } else { crate::cut_list_text(&list, drawing.as_ref().expect("checked").scene()) }, false))
        }
        "movement" => {
            current(drawing)?;
            Ok(text(crate::movement_text(drawing.as_ref().expect("checked")), false))
        }
        "render" => {
            current(drawing)?;
            let view = crate::picture::view(args["view"].as_str().unwrap_or("iso"))?;
            let size = [args["width"].as_u64().unwrap_or(1000).clamp(64, 4096) as u32, args["height"].as_u64().unwrap_or(700).clamp(64, 4096) as u32];
            let png = crate::picture::render(drawing.as_ref().expect("checked"), view, size)?;
            Ok(json!({ "content": [{ "type": "image", "data": base64(&png), "mimeType": "image/png" }], "isError": false }))
        }
        "cut_sheet" => {
            current(drawing)?;
            let to = path()?;
            std::fs::write(to, crate::cut_sheet(drawing.as_ref().expect("checked"))).map_err(|e| e.to_string())?;
            Ok(text(format!("Wrote {}.", to.display()), false))
        }
        "build_sheets" => {
            current(drawing)?;
            let to = path()?;
            let doc = drawing.as_ref().expect("checked");
            let list = cutlist::cut_list(doc.scene());
            std::fs::write(to, woodsketch_doc::buildsheet::to_html(doc.scene(), &list, "Build sheets")).map_err(|e| e.to_string())?;
            Ok(text(format!("Wrote {}.", to.display()), false))
        }
        "export" => {
            current(drawing)?;
            let to = path()?;
            crate::export_to(drawing.as_ref().expect("checked"), to, args["millimeters"].as_bool().unwrap_or(false))?;
            Ok(text(format!("Wrote {}.", to.display()), false))
        }
        other => Err(format!("No tool \"{other}\".")),
    })();
    result.unwrap_or_else(|e| text(e, true))
}

/// Standard base64, for image content.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> shift & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(drawing: &mut Option<Document>, message: Value) -> Value {
        handle(&message.to_string(), drawing).expect("an answer")
    }

    #[test]
    fn an_agent_initializes_lists_tools_and_builds_a_drawing() {
        let mut drawing = None;
        let init = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
        assert_eq!(init["result"]["serverInfo"]["name"], "woodsketch");
        assert!(handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string(), &mut drawing).is_none(), "no answer to a notification");
        let list = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }));
        let names: Vec<&str> = list["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"build") && names.contains(&"render") && names.contains(&"guide"), "{names:?}");

        let guide = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "guide", "arguments": {} } }));
        assert!(guide["result"]["content"][0]["text"].as_str().unwrap().contains("\"parts\""));

        let recipe = json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [
                { "name": "Side", "material": "oak", "size": [30, 11.25, 0.75], "at": [0, 0, 0], "length": "z", "width": "y" },
                { "name": "Shelf", "material": "oak", "size": [20, 11.25, 0.75], "at": [0.75, 0, 12] }
            ],
            "joints": [{ "kind": "let_in", "part": "Shelf", "into": "Side" }]
        });
        let built = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": { "name": "build", "arguments": { "recipe": recipe } } }));
        assert_eq!(built["result"]["isError"], false, "{built}");
        assert!(built["result"]["content"][0]["text"].as_str().unwrap().contains("Red Oak"));
        let cut = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": { "name": "cut_list", "arguments": { "format": "csv" } } }));
        assert!(cut["result"]["content"][0]["text"].as_str().unwrap().contains("Shelf"));

        // A mistake comes back as a tool error naming the item.
        let bad = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": { "name": "build", "arguments": { "recipe": { "parts": [{ "name": "Shelf", "size": [20, 10, 0.75], "material": "teak" }] } } } }));
        assert_eq!(bad["result"]["isError"], true);
        assert!(bad["result"]["content"][0]["text"].as_str().unwrap().contains("parts[0] (Shelf): no material \"teak\""));
        let unknown = ask(&mut drawing, json!({ "jsonrpc": "2.0", "id": 7, "method": "resources/list" }));
        assert_eq!(unknown["error"]["code"], -32601);
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }
}
