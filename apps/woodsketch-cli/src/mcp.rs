//! `woodsketch-cli mcp`: WoodSketch as a Model Context Protocol server on stdin/stdout, so Claude
//! and other assistants can design with it. One drawing is kept between calls, with its recipe:
//! build it from a recipe (or open a file, read back as one), change it a piece at a time (add,
//! change or remove a part, a joint, a profile), check it (summary, cut list, wood movement, a
//! picture), and save it or write the shop sheets. Messages are JSON-RPC 2.0, one per line. The
//! guide and the recipe's JSON Schema are resources too, and there's a prompt to start a design.

use std::io::{BufRead, Write};
use std::path::Path;

use serde_json::{Value, json};
use woodsketch_doc::recipe::{self, JointSpec, PartSpec, ProfileSpec, Recipe};
use woodsketch_doc::{Document, cutlist};

/// The protocol version answered when a client asks for one this doesn't know.
const PROTOCOL: &str = "2025-06-18";

/// What's kept between calls: the drawing, and the recipe that builds it.
#[derive(Default)]
pub struct Session {
    pub drawing: Option<Document>,
    pub recipe: Option<Recipe>,
    /// The drawing's file, once `open` or `save` has named one. Knowing it is what makes
    /// watching work: each edit is written straight back, and the app reloads what it sees.
    pub path: Option<std::path::PathBuf>,
    /// Exactly what was last written to (or read from) `path`. Compared against the file before
    /// an edit overwrites it, so work done in WoodSketch meanwhile isn't thrown away. The text
    /// rather than a timestamp, so saving an unchanged drawing in the app doesn't look like an
    /// edit, and so no clock is involved.
    written: Option<String>,
}

/// Why this session mustn't write to its file, if it mustn't: someone has changed it since the
/// session last wrote or read it. `None` means writing is safe (or there's nothing to compare).
fn changed_underneath(session: &Session) -> Option<String> {
    let path = session.path.as_deref()?;
    let ours = session.written.as_deref()?;
    match std::fs::read_to_string(path) {
        // The same bytes: ours, or saved again in WoodSketch without changing anything.
        Ok(theirs) if theirs == ours => None,
        Ok(_) => Some(format!(
            "{} has changed since this session last wrote it, so something else (WoodSketch, most likely) has edited the drawing. Nothing was changed here, so neither version is lost yet. Use `open` on it to take those changes and carry on from them, or `save` to that path to overwrite them with this session's drawing.",
            path.display()
        )),
        Err(e) => Some(format!(
            "{} can't be read ({e}), so this session won't write over whatever is there. Use `save` to that path to write this session's drawing, or `open` another file.",
            path.display()
        )),
    }
}

/// Whether `path` is a WoodSketch drawing, and so somewhere edits can be written back to. A
/// SketchUp model or a recipe `.json` is something to read: writing drawing JSON over a `.skp`
/// would destroy it, which is why the app treats one as a new unsaved drawing as well.
fn is_drawing(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("woodsketch" | "scraft"))
}

/// Save the drawing to `path` and remember what was written, so [`changed_underneath`] can tell
/// later whether it's still ours.
fn save_remembering(session: &mut Session, path: &Path) -> Result<(), String> {
    let doc = session.drawing.as_ref().ok_or("No drawing to save.")?;
    let text = woodsketch_doc::file::to_string(doc.scene());
    woodsketch_doc::file::write_safely(path, &text).map_err(|e| format!("Couldn't save {}: {e}", path.display()))?;
    session.written = Some(text);
    Ok(())
}

pub fn serve() {
    let mut session = Session::default();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        respond(&line, &mut session, &mut |message| {
            let _ = writeln!(stdout, "{message}");
            let _ = stdout.flush();
        });
    }
}

/// One message in; out, any progress notifications, then the answer (none for a notification).
#[cfg(test)]
fn handle(line: &str, session: &mut Session) -> Vec<Value> {
    let mut out = Vec::new();
    respond(line, session, &mut |message| out.push(message));
    out
}

/// [`handle`], sending each message as it's ready (progress while a build runs).
pub fn respond(line: &str, session: &mut Session, send: &mut dyn FnMut(Value)) {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return send(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "Not JSON" } }));
    };
    let Some(id) = message.get("id").cloned() else { return };
    let method = message["method"].as_str().unwrap_or("");
    let params = &message["params"];
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": params["protocolVersion"].as_str().unwrap_or(PROTOCOL),
            "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
            "serverInfo": { "name": "woodsketch", "version": env!("CARGO_PKG_VERSION") },
            "instructions": "WoodSketch designs woodworking projects. Call `guide` first: it explains the project recipe (JSON) that `build` takes (its JSON Schema is the resource woodsketch://recipe-schema). Build, check with `summary`, `movement` and `render`, then change it a piece at a time (`add_part`, `change_part`, `remove`, `add_joint`, `add_profile`; `recipe` shows what it is now), and `save` or write `cut_sheet` / `build_sheets` for the shop."
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => {
            // Progress for a client that asked: started, each step of a build, done.
            let token = params["_meta"]["progressToken"].clone();
            let mut progress = |done: usize, total: usize, what: &str| {
                if !token.is_null() {
                    send(json!({ "jsonrpc": "2.0", "method": "notifications/progress", "params": { "progressToken": token, "progress": done, "total": total, "message": what } }));
                }
            };
            progress(0, 1, "Working");
            let mut last = 1;
            let result = call(params["name"].as_str().unwrap_or(""), &params["arguments"], session, &mut |done, total, what| {
                last = total.max(1);
                progress(done, last, &format!("Building: {what}"));
            });
            progress(last, last, "Done");
            Ok(result)
        }
        "resources/list" => Ok(json!({ "resources": [
            { "uri": "woodsketch://guide", "name": "guide", "title": "How to write a WoodSketch recipe", "mimeType": "text/markdown" },
            { "uri": "woodsketch://recipe-schema", "name": "recipe-schema", "title": "The recipe's JSON Schema", "mimeType": "application/schema+json" },
            { "uri": "woodsketch://recipe", "name": "recipe", "title": "The current drawing's recipe", "mimeType": "application/json" }
        ] })),
        "resources/read" => match params["uri"].as_str().unwrap_or("") {
            uri @ "woodsketch://guide" => Ok(json!({ "contents": [{ "uri": uri, "mimeType": "text/markdown", "text": crate::GUIDE }] })),
            uri @ "woodsketch://recipe-schema" => Ok(json!({ "contents": [{ "uri": uri, "mimeType": "application/schema+json", "text": recipe::schema().to_string() }] })),
            uri @ "woodsketch://recipe" => match &session.recipe {
                Some(r) => Ok(json!({ "contents": [{ "uri": uri, "mimeType": "application/json", "text": serde_json::to_string_pretty(r).unwrap_or_default() }] })),
                None => Err(json!({ "code": -32002, "message": "No drawing yet: build one from a recipe, or open a file." })),
            },
            other => Err(json!({ "code": -32002, "message": format!("No resource {other}") })),
        },
        "prompts/list" => Ok(json!({ "prompts": [{
            "name": "design",
            "title": "Design a woodworking project",
            "description": "Design a piece of furniture in WoodSketch: a recipe built, checked and refined, then the shop sheets.",
            "arguments": [
                { "name": "project", "description": "What to make, like \"a cherry nightstand with one drawer\"", "required": true },
                { "name": "size", "description": "Overall size, if it matters (\"24 x 16 x 26 inches\")", "required": false }
            ]
        }] })),
        "prompts/get" => {
            let project = params["arguments"]["project"].as_str().unwrap_or("a small bookcase");
            let size = params["arguments"]["size"].as_str().map(|s| format!(" about {s}")).unwrap_or_default();
            Ok(json!({ "description": "Design a woodworking project", "messages": [{ "role": "user", "content": { "type": "text", "text": format!(
                "Design {project}{size} with WoodSketch. Read the `guide` tool first. Build it as a recipe with real joinery (dados, mortise and tenons, dovetails where they suit), check it with `summary`, `movement` and `render`, fix any problems or wood movement warnings, then change it piece by piece until it's right. Finish by writing the cut sheet and build sheets, and tell me what to buy."
            ) } }] }))
        }
        other => Err(json!({ "code": -32601, "message": format!("No method {other}") })),
    };
    send(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    });
}

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({ "name": name, "description": description, "inputSchema": { "type": "object", "properties": properties, "required": required } })
}

fn tools() -> Vec<Value> {
    let path = json!({ "type": "string", "description": "A file path" });
    let name = json!({ "type": "string", "description": "A part's name (or a copy's, template's or group's)" });
    let schema = recipe::schema();
    let def = |d: &str| schema["$defs"][d].clone();
    vec![
        tool("guide", "How to write a WoodSketch project recipe, with an example. Read this before build.", json!({}), &[]),
        tool(
            "build",
            "Build the drawing from a project recipe (see guide). Replaces the current drawing. Returns a summary (size, parts by material, joints, wood movement) or every problem found, each naming the recipe item to fix.",
            json!({ "recipe": { "type": "object", "description": "The project recipe (JSON object, or a string of JSON)" } }),
            &["recipe"],
        ),
        tool("recipe", "The current drawing as a recipe (built from one, or read back from a file opened), with notes on anything a recipe can't say.", json!({}), &[]),
        tool("add_part", "Add a part to the drawing (a board as a box, as in the recipe's parts) and rebuild.", json!({ "part": def("PartSpec") }), &["part"]),
        tool(
            "change_part",
            "Change some of a part's fields (like {\"at\": [0, 0, 12]} or {\"size\": [30, 10, 0.75]}; a new name is followed by its joints) and rebuild.",
            json!({ "name": name, "changes": { "type": "object" } }),
            &["name", "changes"],
        ),
        tool("remove", "Take out a part (with its copies), template or group, and the joints and profiles on it, and rebuild.", json!({ "name": name }), &["name"]),
        tool("add_joint", "Add a joint between two parts (as in the recipe's joints) and rebuild.", json!({ "joint": def("JointSpec") }), &["joint"]),
        tool(
            "remove_joint",
            "Take out the joints of `part` into `into` (of one `kind`, or all) and rebuild.",
            json!({ "part": name, "into": name, "kind": { "type": "string" } }),
            &["part", "into"],
        ),
        tool("add_profile", "Add an edge profile to a part (as in the recipe's profiles) and rebuild.", json!({ "profile": def("ProfileSpec") }), &["profile"]),
        tool("open", "Open a .woodsketch drawing, a recipe .json, or a SketchUp 2021+ .skp as the current drawing (read back as a recipe to change).", json!({ "path": path }), &["path"]),
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
            "A picture of the current drawing (PNG), zoomed to fit: from a standard view or any angle, light or dark, parts picked out.",
            json!({
                "view": { "type": "string", "enum": ["iso", "top", "front", "right"] },
                "angle": { "type": "array", "items": { "type": "number" }, "description": "[around, up] in degrees: around from the front toward the right, up from level" },
                "dark": { "type": "boolean" },
                "highlight": { "type": "array", "items": { "type": "string" }, "description": "Parts or groups to pick out, by name (inside groups too)" },
                "width": { "type": "integer" },
                "height": { "type": "integer" }
            }),
            &[],
        ),
        tool("cut_sheet", "Write the printable cut sheet (parts, what to buy, cutting diagrams) as an HTML file.", json!({ "path": path }), &["path"]),
        tool("build_sheets", "Write the build sheets (a page per part with its steps, then assembly) as an HTML file.", json!({ "path": path }), &["path"]),
        tool("export", "Export the current drawing as .obj (with .mtl) or .stl for other 3D programs, or as .dxf or .svg: each part flattened for CNC (its outline, through holes, and pockets on layers by depth; what a router can't cut from a face is listed, not drawn).", json!({ "path": path, "millimeters": { "type": "boolean" } }), &["path"]),
    ]
}

/// Text content, marked as an error or not.
fn text(text: impl Into<String>, error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text.into() }], "isError": error })
}

/// A recipe item from the arguments: an object, or a string of JSON.
fn item<T: serde::de::DeserializeOwned>(value: &Value, what: &str) -> Result<T, String> {
    let value = match value {
        Value::String(s) => serde_json::from_str(s).map_err(|e| format!("The {what} isn't JSON: {e}"))?,
        Value::Null => return Err(format!("Give a \"{what}\".")),
        other => other.clone(),
    };
    serde_json::from_value(value).map_err(|e| format!("The {what} doesn't fit the recipe format: {e}"))
}

fn call(name: &str, args: &Value, session: &mut Session, progress: &mut dyn FnMut(usize, usize, &str)) -> Value {
    let path = || args["path"].as_str().map(Path::new).ok_or("Give a \"path\".".to_owned());
    let result: Result<Value, String> = (|| {
        // Edits: change the recipe and rebuild, or leave both as they were.
        let mut edit = |change: &dyn Fn(&mut Recipe) -> Result<String, String>| -> Result<Value, String> {
            let mut recipe = session.recipe.clone().ok_or("No drawing yet: build one from a recipe, or open a file.")?;
            // Checked before anything changes: an edit that would write over someone else's work
            // is refused whole, leaving this session and the file as they were. The drawing
            // follows its file in both directions, so this is the other side of that.
            if let Some(theirs) = changed_underneath(session) {
                return Err(theirs);
            }
            let said = change(&mut recipe)?;
            let doc = recipe::build_reporting(&recipe, &mut *progress).map_err(|errors| format!("That doesn't build, so nothing changed:\n- {}", errors.join("\n- ")))?;
            let summary = recipe::summary(&doc);
            (session.drawing, session.recipe) = (Some(doc), Some(recipe));
            // Write it back to its file, so the app beside you shows the change. Nothing to
            // write to until `open` or `save` has named one.
            let saved = match session.path.clone() {
                Some(path) => match save_remembering(session, &path) {
                    Ok(()) => String::new(),
                    // The edit itself worked, so say the file didn't take it rather than fail.
                    Err(e) => format!("\n(Changed here, but couldn't be saved: {e})"),
                },
                None => String::new(),
            };
            Ok(text(format!("{said}\n{summary}{saved}"), false))
        };
        match name {
            "add_part" => {
                let part: PartSpec = item(&args["part"], "part")?;
                return edit(&|r| r.add_part(part.clone(), false).map(|()| format!("Added {}.", part.name)));
            }
            "change_part" => {
                let who = args["name"].as_str().ok_or("Give the part's \"name\".")?.to_owned();
                let changes = args["changes"].clone();
                return edit(&|r| r.change_part(&who, &changes).map(|()| format!("Changed {who}.")));
            }
            "remove" => {
                let who = args["name"].as_str().ok_or("Give the \"name\" to take out.")?.to_owned();
                return edit(&|r| r.remove(&who).map(|()| format!("Took out {who}.")));
            }
            "add_joint" => {
                let joint: JointSpec = item(&args["joint"], "joint")?;
                return edit(&|r| {
                    r.joints.push(joint.clone());
                    Ok(format!("Added a {} of {} into {}.", joint.kind, joint.part, joint.into))
                });
            }
            "remove_joint" => {
                let (part, into) = (args["part"].as_str().ok_or("Give the \"part\".")?.to_owned(), args["into"].as_str().ok_or("Give \"into\".")?.to_owned());
                let kind = args["kind"].as_str().map(str::to_owned);
                return edit(&|r| r.remove_joint(&part, &into, kind.as_deref()).map(|n| format!("Took out {n} joint(s).")));
            }
            "add_profile" => {
                let profile: ProfileSpec = item(&args["profile"], "profile")?;
                return edit(&|r| {
                    r.profiles.push(profile.clone());
                    Ok(format!("Added a {} on {}.", profile.profile, profile.part))
                });
            }
            _ => {}
        }
        let current = || session.drawing.as_ref().ok_or("No drawing yet: build one from a recipe, or open a file.".to_owned());
        match name {
            "guide" => Ok(text(crate::GUIDE, false)),
            "build" => {
                let json = match &args["recipe"] {
                    Value::String(s) => s.clone(),
                    Value::Null => return Err("Give a \"recipe\".".into()),
                    other => other.to_string(),
                };
                let parsed = recipe::parse(&json)?;
                let doc = recipe::build_reporting(&parsed, &mut *progress).map_err(|errors| format!("The recipe has {} problem(s):
- {}", errors.len(), errors.join("
- ")))?;
                let summary = recipe::summary(&doc);
                (session.drawing, session.recipe) = (Some(doc), Some(parsed));
                // A different drawing now, so any file the session was writing to is no longer
                // its own: `save` names one again, rather than this quietly overwriting it.
                (session.path, session.written) = (None, None);
                Ok(text(format!("Built.\n{summary}"), false))
            }
            "recipe" => {
                let recipe = session.recipe.as_ref().ok_or("No drawing yet: build one from a recipe, or open a file.")?;
                Ok(text(serde_json::to_string_pretty(recipe).map_err(|e| e.to_string())?, false))
            }
            "open" => {
                let opened = path()?;
                let doc = crate::load(opened)?;
                let summary = recipe::summary(&doc);
                let (read, notes) = woodsketch_doc::recipe_of::from_document(&doc);
                let notes = if notes.is_empty() { String::new() } else { format!("\nRead back as a recipe, but:\n- {}", notes.join("\n- ")) };
                (session.drawing, session.recipe) = (Some(doc), Some(read));
                // Only a WoodSketch drawing becomes the file this session writes to. A SketchUp
                // model or a recipe is a source, not a destination.
                let following = if is_drawing(opened) {
                    session.path = Some(opened.to_owned());
                    session.written = std::fs::read_to_string(opened).ok();
                    "\nChanges from here on are saved to it as they're made, so the WoodSketch app will show them as you work."
                } else {
                    session.path = None;
                    session.written = None;
                    "\nThis isn't a WoodSketch drawing, so it won't be written to: `save` the drawing somewhere to keep it (and to have the app follow your changes)."
                };
                Ok(text(format!("Opened.\n{summary}{notes}{following}"), false))
            }
            "save" => {
                let to = path()?;
                let first = session.path.as_deref() != Some(to);
                // Asking to save is saying where the drawing belongs, so this writes even when
                // the file changed underneath: it's the deliberate way to overwrite, and the
                // only one. `written` is taken from what goes out, so edits can follow again.
                let overwrote = !first && changed_underneath(session).is_some();
                session.path = Some(to.to_owned());
                save_remembering(session, to)?;
                // Worth saying once, so an assistant can pass it on to whoever it's working for.
                let following = if first { " Changes from here on are saved to it as they're made, so the WoodSketch app will show them as you work." } else { "" };
                let over = if overwrote { " This replaced changes something else had made to that file." } else { "" };
                Ok(text(format!("Saved {}.{following}{over}", to.display()), false))
            }
            "summary" => Ok(text(recipe::summary(current()?), false)),
            "cut_list" => {
                let doc = current()?;
                let list = cutlist::cut_list(doc.scene());
                Ok(text(if args["format"].as_str() == Some("csv") { cutlist::to_csv(&list) } else { crate::cut_list_text(&list, doc.scene()) }, false))
            }
            "movement" => Ok(text(crate::movement_text(current()?), false)),
            "render" => {
                let doc = current()?;
                let angle = match args["angle"].as_array().map(|a| a.iter().filter_map(Value::as_f64).collect::<Vec<_>>()) {
                    Some(a) if a.len() == 2 => Some((a[0] as f32, a[1] as f32)),
                    Some(_) => return Err("An angle is [around, up] in degrees.".into()),
                    None => None,
                };
                let options = crate::picture::Options {
                    view: crate::picture::view(args["view"].as_str().unwrap_or("iso"))?,
                    angle,
                    size: [args["width"].as_u64().unwrap_or(1000).clamp(64, 4096) as u32, args["height"].as_u64().unwrap_or(700).clamp(64, 4096) as u32],
                    dark: args["dark"].as_bool().unwrap_or(false),
                    highlight: args["highlight"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect()).unwrap_or_default(),
                };
                let png = crate::picture::render(doc, &options)?;
                Ok(json!({ "content": [{ "type": "image", "data": base64(&png), "mimeType": "image/png" }], "isError": false }))
            }
            "cut_sheet" => {
                let to = path()?;
                std::fs::write(to, crate::cut_sheet(current()?)).map_err(|e| e.to_string())?;
                Ok(text(format!("Wrote {}.", to.display()), false))
            }
            "build_sheets" => {
                let to = path()?;
                let doc = current()?;
                let list = cutlist::cut_list(doc.scene());
                std::fs::write(to, woodsketch_doc::buildsheet::to_html(doc.scene(), &list, "Build sheets")).map_err(|e| e.to_string())?;
                Ok(text(format!("Wrote {}.", to.display()), false))
            }
            "export" => {
                let to = path()?;
                crate::export_to(current()?, to, args["millimeters"].as_bool().unwrap_or(false))?;
                Ok(text(format!("Wrote {}.", to.display()), false))
            }
            other => Err(format!("No tool \"{other}\".")),
        }
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

    /// The answer to a request (the last message out).
    fn ask(session: &mut Session, message: Value) -> Value {
        handle(&message.to_string(), session).pop().expect("an answer")
    }

    fn call_tool(session: &mut Session, id: u32, name: &str, arguments: Value) -> Value {
        ask(session, json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": arguments } }))["result"].clone()
    }

    fn said(result: &Value) -> &str {
        result["content"][0]["text"].as_str().unwrap_or("")
    }

    #[test]
    fn an_agent_initializes_lists_tools_and_builds_a_drawing() {
        let mut session = Session::default();
        let init = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
        assert_eq!(init["result"]["serverInfo"]["name"], "woodsketch");
        assert!(handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string(), &mut session).is_empty(), "no answer to a notification");
        let list = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }));
        let names: Vec<&str> = list["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"build") && names.contains(&"render") && names.contains(&"guide") && names.contains(&"change_part"), "{names:?}");

        assert!(said(&call_tool(&mut session, 3, "guide", json!({}))).contains("\"parts\""));

        let recipe = json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [
                { "name": "Side", "material": "oak", "size": [30, 11.25, 0.75], "at": [0, 0, 0], "length": "z", "width": "y" },
                { "name": "Shelf", "material": "oak", "size": [20, 11.25, 0.75], "at": [0.75, 0, 12] }
            ],
            "joints": [{ "kind": "let_in", "part": "Shelf", "into": "Side" }]
        });
        let built = call_tool(&mut session, 4, "build", json!({ "recipe": recipe }));
        assert_eq!(built["isError"], false, "{built}");
        assert!(said(&built).contains("Red Oak"));
        assert!(said(&call_tool(&mut session, 5, "cut_list", json!({ "format": "csv" }))).contains("Shelf"));

        // A mistake comes back as a tool error naming the item.
        let bad = call_tool(&mut session, 6, "build", json!({ "recipe": { "parts": [{ "name": "Shelf", "size": [20, 10, 0.75], "material": "teak" }] } }));
        assert_eq!(bad["isError"], true);
        assert!(said(&bad).contains("parts[0] (Shelf): no material \"teak\""));
        let unknown = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 7, "method": "nothing/here" }));
        assert_eq!(unknown["error"]["code"], -32601);
    }

    /// The first material's id in the session's recipe. Reading a drawing back names materials
    /// after themselves, so an id from the recipe that built it may no longer be the one to use.
    fn first_material(session: &Session) -> String {
        session.recipe.as_ref().and_then(|r| r.materials.first()).map(|m| m.id.clone()).expect("a material")
    }

    #[test]
    fn an_edit_wont_write_over_changes_made_in_the_app() {
        // The other side of #6: the app no longer loses the assistant's work, and the assistant
        // mustn't lose the person's. A hand edit saved in WoodSketch is not overwritten silently.
        let dir = std::env::temp_dir().join(format!("woodsketch-mcp-theirs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hutch.woodsketch");

        let mut session = Session::default();
        call_tool(&mut session, 1, "build", json!({ "recipe": {
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [{ "name": "Side", "material": "oak", "size": [30, 11.25, 0.75], "length": "z" }],
        } }));
        call_tool(&mut session, 2, "save", json!({ "path": path.to_string_lossy() }));

        // Someone moves a part by hand in WoodSketch and saves.
        let mut theirs = crate::load(&path).unwrap();
        theirs.edit(|m| m.add_rectangle(glam::DVec3::ZERO, glam::DVec3::X * 9.0, glam::DVec3::Y * 9.0, glam::DVec3::Z));
        woodsketch_doc::file::save(theirs.scene(), &path).unwrap();
        let after_theirs = std::fs::read_to_string(&path).unwrap();

        // The next edit is refused whole, and their file is untouched.
        let shelf = json!({ "name": "Shelf", "material": "oak", "size": [20, 11.25, 0.75] });
        let refused = call_tool(&mut session, 3, "add_part", json!({ "part": shelf }));
        assert_eq!(refused["isError"], true, "{refused}");
        assert!(said(&refused).contains("changed since this session last wrote it"), "{refused}");
        assert!(said(&refused).contains("`open`") && said(&refused).contains("`save`"), "it says how to go on: {refused}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), after_theirs, "their work is still there");
        assert!(!session.recipe.as_ref().unwrap().parts.iter().any(|p| p.name == "Shelf"), "nothing changed here either");

        // `open` takes their changes, and editing follows the file again. Reading a drawing back
        // names its materials after themselves ("Red Oak" -> red_oak), so use what came back.
        call_tool(&mut session, 4, "open", json!({ "path": path.to_string_lossy() }));
        let material = first_material(&session);
        let shelf = json!({ "name": "Shelf", "material": material, "size": [20, 11.25, 0.75] });
        let added = call_tool(&mut session, 5, "add_part", json!({ "part": shelf }));
        assert_eq!(added["isError"], false, "{added}");
        assert_ne!(std::fs::read_to_string(&path).unwrap(), after_theirs, "the edit reached the file");

        // Saving over a changed file is allowed, since asking to save says where it belongs.
        let mut ours = crate::load(&path).unwrap();
        ours.edit(|m| m.add_rectangle(glam::DVec3::ZERO, glam::DVec3::X, glam::DVec3::Y, glam::DVec3::Z));
        woodsketch_doc::file::save(ours.scene(), &path).unwrap();
        let saved = call_tool(&mut session, 6, "save", json!({ "path": path.to_string_lossy() }));
        assert_eq!(saved["isError"], false, "{saved}");
        assert!(said(&saved).contains("replaced changes"), "it says it overwrote: {saved}");
        // And editing works again straight after.
        let more = call_tool(&mut session, 7, "change_part", json!({ "name": "Shelf", "changes": { "size": [22, 11.25, 0.75] } }));
        assert_eq!(more["isError"], false, "{more}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_sketchup_model_or_a_recipe_is_never_written_over() {
        // `open` takes .skp and recipe .json too. Those are sources: writing drawing JSON over a
        // SketchUp model would destroy it, so they don't become the file edits are saved to.
        let dir = std::env::temp_dir().join(format!("woodsketch-mcp-source-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let recipe_path = dir.join("bench.json");
        let recipe = json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [{ "name": "Top", "material": "oak", "size": [48, 20, 1.5] }],
        });
        std::fs::write(&recipe_path, recipe.to_string()).unwrap();
        let before = std::fs::read_to_string(&recipe_path).unwrap();

        let mut session = Session::default();
        let opened = call_tool(&mut session, 1, "open", json!({ "path": recipe_path.to_string_lossy() }));
        assert_eq!(opened["isError"], false, "{opened}");
        assert_eq!(session.path, None, "a recipe isn't a place to write");
        assert!(said(&opened).contains("`save` the drawing somewhere"), "{opened}");

        let leg = json!({ "name": "Leg", "material": first_material(&session), "size": [28, 3, 3], "length": "z" });
        let added = call_tool(&mut session, 2, "add_part", json!({ "part": leg }));
        assert_eq!(added["isError"], false, "{added}");
        assert_eq!(std::fs::read_to_string(&recipe_path).unwrap(), before, "the recipe file is untouched");

        assert!(is_drawing(Path::new("a.woodsketch")) && is_drawing(Path::new("OLD.SCRAFT")));
        assert!(!is_drawing(Path::new("model.skp")) && !is_drawing(Path::new("r.json")) && !is_drawing(Path::new("plain")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn building_a_new_drawing_lets_go_of_the_old_file() {
        let dir = std::env::temp_dir().join(format!("woodsketch-mcp-rebuild-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("first.woodsketch");
        let mut session = Session::default();
        call_tool(&mut session, 1, "build", json!({ "recipe": { "parts": [{ "name": "A", "size": [10, 10, 1] }] } }));
        call_tool(&mut session, 2, "save", json!({ "path": path.to_string_lossy() }));
        assert!(session.path.is_some());
        let first = std::fs::read_to_string(&path).unwrap();

        // A different project: the old file isn't this drawing's any more.
        call_tool(&mut session, 3, "build", json!({ "recipe": { "parts": [{ "name": "B", "size": [20, 20, 2] }] } }));
        assert_eq!(session.path, None, "it let go of the file");
        assert_eq!(call_tool(&mut session, 4, "add_part", json!({ "part": { "name": "C", "size": [5, 5, 1] } }))["isError"], false);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), first, "the first project's file is untouched");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_edit_is_written_back_to_the_file_so_the_app_can_follow() {
        // Issue #6: the drawing on screen never saw the assistant's changes. Once a file has
        // been named, each edit is saved to it, and the app (watching the file) reloads.
        let dir = std::env::temp_dir().join(format!("woodsketch-mcp-save-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bench.woodsketch");

        let mut session = Session::default();
        let recipe = json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [{ "name": "Top", "material": "oak", "size": [60, 24, 1.5] }],
        });
        call_tool(&mut session, 1, "build", json!({ "recipe": recipe }));
        assert_eq!(session.path, None, "nowhere to save to yet");

        // Building alone writes nothing: there is no file until one is named.
        assert!(!path.exists());
        let parts = |p: &std::path::Path| {
            let scene = woodsketch_doc::file::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
            woodsketch_doc::Document::from_scene(scene).view().model().faces().count()
        };

        let saved = call_tool(&mut session, 2, "save", json!({ "path": path.to_string_lossy() }));
        assert_eq!(saved["isError"], false, "{saved}");
        assert!(said(&saved).contains("saved here as they're made") || said(&saved).contains("as they're made"), "{saved}");
        assert_eq!(session.path.as_deref(), Some(path.as_path()), "the file is remembered");
        let one_part = parts(&path);

        // An edit, with no save call: it lands in the file by itself.
        let leg = json!({ "name": "Leg", "material": "oak", "size": [30, 3, 3], "length": "z" });
        let added = call_tool(&mut session, 3, "add_part", json!({ "part": leg }));
        assert_eq!(added["isError"], false, "{added}");
        assert!(parts(&path) > one_part, "the new part reached the file without asking");

        // An edit that doesn't build leaves the file alone, as it leaves the drawing alone.
        let before = std::fs::read_to_string(&path).unwrap();
        let broken = call_tool(&mut session, 4, "add_joint", json!({ "joint": { "kind": "dado", "part": "Leg", "into": "Nothing" } }));
        assert_eq!(broken["isError"], true);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "a failed edit writes nothing");

        // `open` names the file too, so changes to something opened follow it back.
        let mut next = Session::default();
        call_tool(&mut next, 1, "open", json!({ "path": path.to_string_lossy() }));
        assert_eq!(next.path.as_deref(), Some(path.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn saving_says_nothing_on_stdout_because_that_is_the_protocol() {
        // `woodsketch-cli save` prints what it wrote; the MCP server must not, since its stdout
        // carries JSON-RPC. The answer goes in the reply instead.
        let dir = std::env::temp_dir().join(format!("woodsketch-mcp-quiet-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quiet.woodsketch");
        let mut session = Session::default();
        call_tool(&mut session, 1, "build", json!({ "recipe": { "parts": [{ "name": "Shelf", "size": [20, 10, 0.75] }] } }));
        let out = handle(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": { "name": "save", "arguments": { "path": path.to_string_lossy() } } }).to_string(), &mut session);
        // Every message out is JSON-RPC; nothing stray rides along.
        for message in &out {
            assert_eq!(message["jsonrpc"], "2.0", "{message}");
        }
        assert!(path.exists(), "it did save");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_agent_changes_the_drawing_a_piece_at_a_time() {
        let mut session = Session::default();
        let recipe = json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [
                { "name": "Side", "material": "oak", "size": [30, 11.25, 0.75], "length": "z", "width": "y" },
                { "name": "Shelf", "material": "oak", "size": [20, 11.25, 0.75], "at": [0.75, 0, 12] }
            ]
        });
        call_tool(&mut session, 1, "build", json!({ "recipe": recipe }));
        // A second shelf, a joint, the first shelf moved up, a profile.
        let shelf = json!({ "name": "Shelf 2", "material": "oak", "size": [20, 11.25, 0.75], "at": [0.75, 0, 24] });
        assert_eq!(call_tool(&mut session, 2, "add_part", json!({ "part": shelf }))["isError"], false);
        let joint = call_tool(&mut session, 3, "add_joint", json!({ "joint": { "kind": "dado", "part": "Shelf 2", "into": "Side" } }));
        assert!(said(&joint).contains("Joints: 1 dado"), "{joint}");
        assert_eq!(call_tool(&mut session, 4, "change_part", json!({ "name": "Shelf", "changes": { "at": [0.75, 0, 6] } }))["isError"], false);
        let profile = call_tool(&mut session, 5, "add_profile", json!({ "profile": { "part": "Shelf", "profile": "chamfer", "size": 0.125, "around": "top" } }));
        assert_eq!(profile["isError"], false, "{profile}");
        let now = call_tool(&mut session, 6, "recipe", json!({}));
        assert!(said(&now).contains("\"Shelf 2\"") && said(&now).contains("chamfer"));
        // An edit that doesn't build changes nothing.
        let broken = call_tool(&mut session, 7, "add_joint", json!({ "joint": { "kind": "dado", "part": "Shelf", "into": "Nothing" } }));
        assert_eq!(broken["isError"], true);
        assert!(said(&broken).contains("nothing changed"));
        assert_eq!(session.recipe.as_ref().unwrap().joints.len(), 1);
        assert_eq!(call_tool(&mut session, 8, "remove", json!({ "name": "Shelf 2" }))["isError"], false);
        assert!(session.recipe.as_ref().unwrap().joints.is_empty(), "its joint went with it");
    }

    #[test]
    fn resources_prompts_and_progress() {
        let mut session = Session::default();
        let list = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/list" }));
        assert_eq!(list["result"]["resources"].as_array().unwrap().len(), 3);
        let schema = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 2, "method": "resources/read", "params": { "uri": "woodsketch://recipe-schema" } }));
        assert!(schema["result"]["contents"][0]["text"].as_str().unwrap().contains("PartSpec"));
        let guide = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 3, "method": "resources/read", "params": { "uri": "woodsketch://guide" } }));
        assert!(guide["result"]["contents"][0]["text"].as_str().unwrap().contains("recipe"));
        let prompt = ask(&mut session, json!({ "jsonrpc": "2.0", "id": 4, "method": "prompts/get", "params": { "name": "design", "arguments": { "project": "a walnut side table" } } }));
        assert!(prompt["result"]["messages"][0]["content"]["text"].as_str().unwrap().contains("a walnut side table"));
        // Progress, for a client that asks for it, before the answer.
        let out = handle(&json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": { "name": "guide", "arguments": {}, "_meta": { "progressToken": "p1" } } }).to_string(), &mut session);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0]["method"], "notifications/progress");
        assert_eq!(out[2]["id"], 5);
        // A build says each step: two parts and a joint.
        let recipe = json!({ "parts": [
            { "name": "Side", "size": [30, 12, 0.75], "length": "z", "width": "y" },
            { "name": "Shelf", "size": [20, 12, 0.75], "at": [0.75, 0, 10] }
        ], "joints": [{ "kind": "let_in", "part": "Shelf", "into": "Side" }] });
        let out = handle(&json!({ "jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": { "name": "build", "arguments": { "recipe": recipe }, "_meta": { "progressToken": "p2" } } }).to_string(), &mut session);
        let steps: Vec<(u64, u64, &str)> = out[..out.len() - 1].iter().map(|m| (m["params"]["progress"].as_u64().unwrap(), m["params"]["total"].as_u64().unwrap(), m["params"]["message"].as_str().unwrap())).collect();
        assert_eq!(steps.len(), 5, "{steps:?}");
        assert_eq!(steps[3], (2, 3, "Building: joints[0] (let_in, Shelf into Side)"));
        assert_eq!(steps[4], (3, 3, "Done"));
        assert_eq!(out[5]["result"]["isError"], false, "{}", out[5]);
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
