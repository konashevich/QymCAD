//! The process that holds one session and speaks the protocol.
//!
//! The session stays on this thread. Constructing it writes nothing, so standard output stays
//! protocol. `initialize` is answered with revision `2025-11-25`. `notifications/initialized`
//! gets no answer. `tools/list` names the fifty-two tools. `tools/call` runs that tool on the session.
//! A success reply is one line: `structuredContent` holds the numbers, and the text block is that
//! same compact JSON, so the line contains no second line. A refusal the session returns is
//! `isError` with that text, and the process stays up for the next request. `document.read`
//! includes each rebuild refusal. When the input closes, the process exits.

use std::io::{BufRead, Write};

fn main() {
    let mut session = qymcad_drive::Session::empty();
    let incoming = std::io::stdin();
    let mut outgoing = std::io::stdout().lock();
    for read in incoming.lock().lines() {
        let Ok(line) = read else { break };
        if line.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        match msg.get("method").and_then(serde_json::Value::as_str) {
            Some("initialize") => send(&mut outgoing, &msg, greeted()),
            Some("tools/list") => send(&mut outgoing, &msg, listed()),
            Some("tools/call") => {
                if let Some(result) = called(&mut session, &msg) {
                    send(&mut outgoing, &msg, result);
                }
            }
            Some("notifications/initialized") => {}
            _ => {}
        }
    }
}

fn greeted() -> serde_json::Value {
    serde_json::json!({
        "protocolVersion": "2025-11-25",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "qymcad", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn listed() -> serde_json::Value {
    // The json! macro recurses once per tool. Fifty-two tools in one array exceed its recursion limit, so the list is three arrays joined in order.
    let mut tools = taken(sketch_through_shell());
    tools.extend(taken(sphere_through_mirror()));
    tools.extend(taken(offset_through_save()));
    serde_json::json!({ "tools": tools })
}

fn taken(value: serde_json::Value) -> Vec<serde_json::Value> {
    match value {
        serde_json::Value::Array(tools) => tools,
        _ => panic!("the tools"),
    }
}

fn sketch_through_shell() -> serde_json::Value {
    serde_json::json!([
            {
                "name": "sketch.datum",
                "description": "Draw a rectangle on a datum plane.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "plane": { "type": "number" },
                        "from": { "type": "object" },
                        "to": { "type": "object" }
                    },
                    "required": ["plane", "from", "to"]
                }
            },
            {
                "name": "sketch.line",
                "description": "Draw an open line on a coordinate plane.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "plane": { "type": "string" },
                        "from": { "type": "object" },
                        "to": { "type": "object" }
                    },
                    "required": ["plane", "from", "to"]
                }
            },
            {
                "name": "sketch.plane",
                "description": "Raise a datum plane off a coordinate plane.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "plane": { "type": "string" },
                        "offset": { "type": "number" }
                    },
                    "required": ["plane", "offset"]
                }
            },
            {
                "name": "sketch.rectangle",
                "description": "Draw a rectangle on a coordinate plane.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "plane": { "type": "string" },
                        "from": { "type": "object" },
                        "to": { "type": "object" }
                    },
                    "required": ["plane", "from", "to"]
                }
            },
            {
                "name": "sketch.resize",
                "description": "Move a rectangle to new corners.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sketch": { "type": "number" },
                        "contour": { "type": "number" },
                        "from": { "type": "object" },
                        "to": { "type": "object" }
                    },
                    "required": ["sketch", "contour", "from", "to"]
                }
            },
            {
                "name": "part.extrude",
                "description": "Extrude a closed contour by a height.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sketch": { "type": "number" },
                        "contour": { "type": "number" },
                        "height": { "type": "number" }
                    },
                    "required": ["sketch", "contour", "height"]
                }
            },
            {
                "name": "part.cuboid",
                "description": "Build a box from its three sizes.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "length": { "type": "number" },
                        "width": { "type": "number" },
                        "height": { "type": "number" }
                    },
                    "required": ["length", "width", "height"]
                }
            },
            {
                "name": "part.cylinder",
                "description": "Build a cylinder from its radius and height.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "radius": { "type": "number" },
                        "height": { "type": "number" }
                    },
                    "required": ["radius", "height"]
                }
            },
            {
                "name": "part.draft",
                "description": "Tilt one wall about a neutral face.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "wall": { "type": "number" },
                        "neutral": { "type": "number" },
                        "angle": { "type": "number" }
                    },
                    "required": ["body", "wall", "neutral", "angle"]
                }
            },
            {
                "name": "part.cone",
                "description": "Build a cone from its two radii and its height.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "bottom": { "type": "number" },
                        "top": { "type": "number" },
                        "height": { "type": "number" }
                    },
                    "required": ["bottom", "top", "height"]
                }
            },
            {
                "name": "part.copy_face",
                "description": "Copy one face out as a sheet.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" }
                    },
                    "required": ["body", "face"]
                }
            },
            {
                "name": "part.bind",
                "description": "Bind an extrusion height to an expression.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "dimension": { "type": "string" },
                        "expression": { "type": "string" }
                    },
                    "required": ["body", "dimension", "expression"]
                }
            },
            {
                "name": "part.boolean",
                "description": "Cut one body out of another.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "base": { "type": "number" },
                        "op": { "type": "string" },
                        "tool": { "type": "number" }
                    },
                    "required": ["base", "op", "tool"]
                }
            },
            {
                "name": "part.fillet",
                "description": "Round one edge of a body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "edge": { "type": "number" },
                        "radius": { "type": "number" }
                    },
                    "required": ["body", "edge", "radius"]
                }
            },
            {
                "name": "part.chamfer",
                "description": "Bevel one edge of a body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "edge": { "type": "number" },
                        "distance": { "type": "number" }
                    },
                    "required": ["body", "edge", "distance"]
                }
            },
            {
                "name": "part.circular",
                "description": "Turn a body about the vertical axis.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "copies": { "type": "number" },
                        "angle": { "type": "number" }
                    },
                    "required": ["body", "copies", "angle"]
                }
            },
            {
                "name": "part.shell",
                "description": "Open one face of a body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" },
                        "thickness": { "type": "number" }
                    },
                    "required": ["body", "face", "thickness"]
                }
            }
        ]
    )
}

fn sphere_through_mirror() -> serde_json::Value {
    serde_json::json!([
            {
                "name": "part.sphere",
                "description": "Build a sphere from its radius.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "radius": { "type": "number" }
                    },
                    "required": ["radius"]
                }
            },
            {
                "name": "part.split",
                "description": "Split a body with the table.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "plane": { "type": "string" },
                        "offset": { "type": "number" }
                    },
                    "required": ["body", "plane", "offset"]
                }
            },
            {
                "name": "part.split_faces",
                "description": "Mark the faces of a body with the table.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "plane": { "type": "string" },
                        "offset": { "type": "number" }
                    },
                    "required": ["body", "plane", "offset"]
                }
            },
            {
                "name": "part.stitch",
                "description": "Sew sheets into one body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sheets": { "type": "array", "items": { "type": "number" } },
                        "gap": { "type": "number" }
                    },
                    "required": ["sheets", "gap"]
                }
            },
            {
                "name": "part.sweep",
                "description": "Sweep a closed section along an open path.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "section": {
                            "type": "object",
                            "properties": {
                                "sketch": { "type": "number" },
                                "contour": { "type": "number" }
                            },
                            "required": ["sketch", "contour"]
                        },
                        "path": {
                            "type": "object",
                            "properties": {
                                "sketch": { "type": "number" },
                                "contour": { "type": "number" }
                            },
                            "required": ["sketch", "contour"]
                        }
                    },
                    "required": ["section", "path"]
                }
            },
            {
                "name": "part.thicken",
                "description": "Grow a plate on one face.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" },
                        "thickness": { "type": "number" }
                    },
                    "required": ["body", "face", "thickness"]
                }
            },
            {
                "name": "part.thread",
                "description": "Cut a metric thread on a circular rim.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "rim": { "type": "number" },
                        "diameter": { "type": "number" },
                        "pitch": { "type": "number" },
                        "length": { "type": "number" }
                    },
                    "required": ["body", "rim", "diameter", "pitch", "length"]
                }
            },
            {
                "name": "part.torus",
                "description": "Build a torus from its ring and its tube.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "ring": { "type": "number" },
                        "tube": { "type": "number" }
                    },
                    "required": ["ring", "tube"]
                }
            },
            {
                "name": "part.trim",
                "description": "Cut a sheet and keep the piece nearest a point.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sheet": { "type": "number" },
                        "tool": { "type": "number" },
                        "keep": {
                            "type": "object",
                            "properties": {
                                "x": { "type": "number" },
                                "y": { "type": "number" },
                                "z": { "type": "number" }
                            },
                            "required": ["x", "y", "z"]
                        }
                    },
                    "required": ["sheet", "tool", "keep"]
                }
            },
            {
                "name": "part.hole",
                "description": "Drill a hole in one face of a body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" },
                        "diameter": { "type": "number" },
                        "depth": { "type": "number" }
                    },
                    "required": ["body", "face", "diameter", "depth"]
                }
            },
            {
                "name": "part.linear",
                "description": "Lay a body out in a straight row.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "step": {
                            "type": "object",
                            "properties": {
                                "x": { "type": "number" },
                                "y": { "type": "number" },
                                "z": { "type": "number" }
                            },
                            "required": ["x", "y", "z"]
                        },
                        "copies": { "type": "number" }
                    },
                    "required": ["body", "step", "copies"]
                }
            },
            {
                "name": "part.loft",
                "description": "Loft a lower section up to an upper section.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "lower": {
                            "type": "object",
                            "properties": {
                                "sketch": { "type": "number" },
                                "contour": { "type": "number" }
                            },
                            "required": ["sketch", "contour"]
                        },
                        "upper": {
                            "type": "object",
                            "properties": {
                                "sketch": { "type": "number" },
                                "contour": { "type": "number" }
                            },
                            "required": ["sketch", "contour"]
                        }
                    },
                    "required": ["lower", "upper"]
                }
            },
            {
                "name": "part.mirror",
                "description": "Mirror a body through the table.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "plane": { "type": "string" }
                    },
                    "required": ["body", "plane"]
                }
            },
            {
                "name": "part.move",
                "description": "Move a body by a placement.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "by": { "type": "array", "items": { "type": "number" } }
                    },
                    "required": ["body", "by"]
                }
            }
        ]
    )
}

fn offset_through_save() -> serde_json::Value {
    serde_json::json!([
            {
                "name": "part.offset_face",
                "description": "Move one face along its normal into a sheet.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" },
                        "distance": { "type": "number" }
                    },
                    "required": ["body", "face", "distance"]
                }
            },
            {
                "name": "part.patch",
                "description": "Span one edge into a sheet.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "edge": { "type": "number" },
                        "meet": { "type": "string" }
                    },
                    "required": ["body", "edge", "meet"]
                }
            },
            {
                "name": "part.prism",
                "description": "Build a prism from its circumradius, its height, and its number of sides.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "radius": { "type": "number" },
                        "height": { "type": "number" },
                        "sides": { "type": "number" }
                    },
                    "required": ["radius", "height", "sides"]
                }
            },
            {
                "name": "part.push",
                "description": "Move one face along its own normal.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" },
                        "distance": { "type": "number" }
                    },
                    "required": ["body", "face", "distance"]
                }
            },
            {
                "name": "part.remove_face",
                "description": "Take one face off a body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" }
                    },
                    "required": ["body", "face"]
                }
            },
            {
                "name": "part.replace_face",
                "description": "Replace one face of a solid with a sheet.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "face": { "type": "number" },
                        "surface": { "type": "number" }
                    },
                    "required": ["body", "face", "surface"]
                }
            },
            {
                "name": "part.revolve",
                "description": "Turn a closed contour about a sketch axis.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sketch": { "type": "number" },
                        "contour": { "type": "number" },
                        "axis": { "type": "string" },
                        "angle": { "type": "number" }
                    },
                    "required": ["sketch", "contour", "axis", "angle"]
                }
            },
            {
                "name": "document.back",
                "description": "Put back the last kept change.",
                "inputSchema": { "type": "object" }
            },
            {
                "name": "document.mate",
                "description": "Fasten the origins of two parts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "stay": { "type": "number" },
                        "bring": { "type": "number" }
                    },
                    "required": ["stay", "bring"]
                }
            },
            {
                "name": "document.open",
                "description": "Read a saved document.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            },
            {
                "name": "document.open_step",
                "description": "Read the solids of a STEP file.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            },
            {
                "name": "document.open_stl",
                "description": "Read an STL file as one mesh.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            },
            {
                "name": "document.parameter",
                "description": "Store a document parameter.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "expression": { "type": "string" }
                    },
                    "required": ["name", "expression"]
                }
            },
            {
                "name": "document.part",
                "description": "Add a part at a placement.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "place": { "type": "array", "items": { "type": "number" } }
                    },
                    "required": ["place"]
                }
            },
            {
                "name": "document.read",
                "description": "Rebuild the document and return its tree.",
                "inputSchema": { "type": "object" }
            },
            {
                "name": "document.save",
                "description": "Write the document to a .qcad file.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            },
            {
                "name": "document.save_step",
                "description": "Write the live bodies to a STEP file.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            },
            {
                "name": "document.save_stl",
                "description": "Write the stored meshes to an STL file.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            },
            {
                "name": "document.show",
                "description": "Open a saved file in a new copy of the program.",
                "inputSchema": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] }
            },
            {
                "name": "document.slide",
                "description": "Fasten the origins of two parts with a slider.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "stay": { "type": "number" },
                        "bring": { "type": "number" },
                        "by": { "type": "number" }
                    },
                    "required": ["stay", "bring", "by"]
                }
            },
            {
                "name": "part.auger",
                "description": "Weld one flight onto a circular rim.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "body": { "type": "number" },
                        "rim": { "type": "number" },
                        "outer": { "type": "number" },
                        "pitch": { "type": "number" },
                        "thickness": { "type": "number" },
                        "length": { "type": "number" }
                    },
                    "required": ["body", "rim", "outer", "pitch", "thickness", "length"]
                }
            }
        ]
    )
}

fn called(session: &mut qymcad_drive::Session, msg: &serde_json::Value) -> Option<serde_json::Value> {
    let name = msg.get("params").and_then(|params| params.get("name")).and_then(serde_json::Value::as_str).unwrap_or("");
    let arguments = msg.get("params").and_then(|params| params.get("arguments")).cloned().unwrap_or_else(|| serde_json::json!({}));
    let outcome = match name {
        "sketch.datum" => datum(session, &arguments),
        "sketch.line" => line(session, &arguments),
        "sketch.plane" => plane(session, &arguments),
        "sketch.rectangle" => rectangle(session, &arguments),
        "sketch.resize" => resize(session, &arguments),
        "part.extrude" => extrude(session, &arguments),
        "part.cuboid" => cuboid(session, &arguments),
        "part.cylinder" => cylinder(session, &arguments),
        "part.draft" => draft(session, &arguments),
        "part.sphere" => sphere(session, &arguments),
        "part.split" => split(session, &arguments),
        "part.split_faces" => split_faces(session, &arguments),
        "part.stitch" => stitch(session, &arguments),
        "part.sweep" => sweep(session, &arguments),
        "part.thicken" => thicken(session, &arguments),
        "part.thread" => thread(session, &arguments),
        "part.torus" => torus(session, &arguments),
        "part.trim" => trim(session, &arguments),
        "part.prism" => prism(session, &arguments),
        "part.push" => push(session, &arguments),
        "part.remove_face" => remove_face(session, &arguments),
        "part.replace_face" => replace_face(session, &arguments),
        "part.revolve" => revolve(session, &arguments),
        "part.cone" => cone(session, &arguments),
        "part.copy_face" => copy_face(session, &arguments),
        "part.bind" => bind(session, &arguments),
        "part.boolean" => boolean(session, &arguments),
        "part.fillet" => fillet(session, &arguments),
        "part.chamfer" => chamfer(session, &arguments),
        "part.circular" => circular(session, &arguments),
        "part.shell" => shell(session, &arguments),
        "part.hole" => hole(session, &arguments),
        "part.linear" => linear(session, &arguments),
        "part.loft" => loft(session, &arguments),
        "part.mirror" => mirror(session, &arguments),
        "part.move" => move_body(session, &arguments),
        "part.offset_face" => offset_face(session, &arguments),
        "part.patch" => patch(session, &arguments),
        "document.back" => back(session),
        "document.mate" => mate(session, &arguments),
        "document.open" => open_document(session, &arguments),
        "document.open_step" => open_step(&arguments),
        "document.open_stl" => open_stl(&arguments),
        "document.parameter" => parameter(session, &arguments),
        "document.part" => part(session, &arguments),
        "document.read" => Ok(read(session)),
        "document.save" => save(session, &arguments),
        "document.save_step" => save_step(session, &arguments),
        "document.save_stl" => save_stl(session, &arguments),
        "document.show" => show(&arguments),
        "document.slide" => slide(session, &arguments),
        "part.auger" => auger(session, &arguments),
        _ => return None,
    };
    Some(match outcome {
        Ok(structured) => {
            // Compact JSON has no newline, so the text block stays inside the one protocol line.
            let text = serde_json::to_string(&structured).expect("the reply");
            serde_json::json!({
                "content": [{ "type": "text", "text": text }],
                "structuredContent": structured,
            })
        }
        // The session's own text. A translated sentence would be a different refusal.
        Err(text) => serde_json::json!({
            "content": [{ "type": "text", "text": text }],
            "isError": true,
        }),
    })
}

fn datum(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let plane = qymcad_drive::PlaneId { id: args.get("plane").and_then(serde_json::Value::as_u64).expect("the plane") };
    let corners = qymcad_drive::Corners { from: qymcad_drive::From { at: corner_of(args, "from") }, to: qymcad_drive::To { at: corner_of(args, "to") } };
    let drawn = session.rectangle_on(plane, corners).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "sketch": drawn.sketch.id, "contour": drawn.contour.id }))
}

fn line(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let plane = plane_of(args.get("plane").and_then(serde_json::Value::as_str).unwrap_or(""));
    let drawn = session.line(plane, qymcad_drive::Segment { from: qymcad_drive::From { at: corner_of(args, "from") }, to: qymcad_drive::To { at: corner_of(args, "to") } }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "sketch": drawn.sketch.id, "contour": drawn.contour.id }))
}

fn plane(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let base = plane_of(args.get("plane").and_then(serde_json::Value::as_str).unwrap_or(""));
    let offset = qymcad_drive::Offset { mm: args.get("offset").and_then(serde_json::Value::as_f64).expect("the offset") };
    let made = session.offset_plane(base, offset).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "plane": made.id }))
}

fn rectangle(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let plane = plane_of(args.get("plane").and_then(serde_json::Value::as_str).unwrap_or(""));
    let drawn =
        session.rectangle(plane, qymcad_drive::Corners { from: qymcad_drive::From { at: corner_of(args, "from") }, to: qymcad_drive::To { at: corner_of(args, "to") } }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "sketch": drawn.sketch.id, "contour": drawn.contour.id }))
}

fn resize(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let sketch = args.get("sketch").and_then(serde_json::Value::as_u64).expect("the sketch");
    let contour = args.get("contour").and_then(serde_json::Value::as_u64).expect("the contour");
    let corners = qymcad_drive::Corners { from: qymcad_drive::From { at: corner_of(args, "from") }, to: qymcad_drive::To { at: corner_of(args, "to") } };
    session.resize(qymcad_drive::Profile { sketch: qymcad_drive::SketchId { id: sketch }, contour: qymcad_drive::ContourId { id: contour } }, corners).map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn extrude(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let sketch = args.get("sketch").and_then(serde_json::Value::as_u64).expect("the sketch");
    let contour = args.get("contour").and_then(serde_json::Value::as_u64).expect("the contour");
    let height = args.get("height").and_then(serde_json::Value::as_f64).expect("the height");
    let made = session
        .extrude(qymcad_drive::Profile { sketch: qymcad_drive::SketchId { id: sketch }, contour: qymcad_drive::ContourId { id: contour } }, qymcad_drive::Height { mm: height })
        .map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn cuboid(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let length = qymcad_drive::BoxLength { mm: args.get("length").and_then(serde_json::Value::as_f64).expect("the length") };
    let width = qymcad_drive::Width { mm: args.get("width").and_then(serde_json::Value::as_f64).expect("the width") };
    let height = qymcad_drive::Height { mm: args.get("height").and_then(serde_json::Value::as_f64).expect("the height") };
    let made = session.cuboid(qymcad_drive::Cuboid { length, width, height }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn cylinder(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let radius = qymcad_drive::Radius { mm: args.get("radius").and_then(serde_json::Value::as_f64).expect("the radius") };
    let height = qymcad_drive::Height { mm: args.get("height").and_then(serde_json::Value::as_f64).expect("the height") };
    let made = session.cylinder(qymcad_drive::Cylinder { radius, height }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn sphere(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let radius = qymcad_drive::Radius { mm: args.get("radius").and_then(serde_json::Value::as_f64).expect("the radius") };
    let made = session.sphere(radius).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn cone(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let bottom = qymcad_drive::BottomRadius { mm: args.get("bottom").and_then(serde_json::Value::as_f64).expect("the bottom") };
    let top = qymcad_drive::TopRadius { mm: args.get("top").and_then(serde_json::Value::as_f64).expect("the top") };
    let height = qymcad_drive::Height { mm: args.get("height").and_then(serde_json::Value::as_f64).expect("the height") };
    let made = session.cone(qymcad_drive::Cone { bottom, top, height }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn copy_face(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let face = qymcad_drive::FaceId { id: u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face") };
    let made = session.copy_face(qymcad_drive::CopyFace { body, face }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn thicken(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let face = qymcad_drive::FaceId { id: u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face") };
    let plate = qymcad_drive::Plate { mm: args.get("thickness").and_then(serde_json::Value::as_f64).expect("the thickness") };
    let made = session.thicken(qymcad_drive::Thicken { body, face, plate }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn thread(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let rim = qymcad_drive::EdgeId { id: u32::try_from(args.get("rim").and_then(serde_json::Value::as_u64).expect("the rim")).expect("the rim") };
    let diameter = qymcad_drive::Nominal { mm: args.get("diameter").and_then(serde_json::Value::as_f64).expect("the diameter") };
    let pitch = qymcad_drive::Pitch { mm: args.get("pitch").and_then(serde_json::Value::as_f64).expect("the pitch") };
    let length = qymcad_drive::ThreadLength { mm: args.get("length").and_then(serde_json::Value::as_f64).expect("the length") };
    let made = session.thread(qymcad_drive::Thread { body, rim, diameter, pitch, length }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn torus(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let ring = qymcad_drive::Ring { mm: args.get("ring").and_then(serde_json::Value::as_f64).expect("the ring") };
    let tube = qymcad_drive::Tube { mm: args.get("tube").and_then(serde_json::Value::as_f64).expect("the tube") };
    let made = session.torus(qymcad_drive::Torus { ring, tube }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn trim(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let sheet = qymcad_drive::BodyId { id: args.get("sheet").and_then(serde_json::Value::as_u64).expect("the sheet") };
    let tool = qymcad_drive::BodyId { id: args.get("tool").and_then(serde_json::Value::as_u64).expect("the tool") };
    let keep = args.get("keep").expect("the keep");
    let keep = qymcad_drive::Keep {
        x: qymcad_drive::KeepX { mm: keep.get("x").and_then(serde_json::Value::as_f64).expect("the x") },
        y: qymcad_drive::KeepY { mm: keep.get("y").and_then(serde_json::Value::as_f64).expect("the y") },
        z: qymcad_drive::KeepZ { mm: keep.get("z").and_then(serde_json::Value::as_f64).expect("the z") },
    };
    let made = session.trim(qymcad_drive::Trim { sheet, keep, tool }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn prism(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let radius = qymcad_drive::Radius { mm: args.get("radius").and_then(serde_json::Value::as_f64).expect("the radius") };
    let height = qymcad_drive::Height { mm: args.get("height").and_then(serde_json::Value::as_f64).expect("the height") };
    let sides = qymcad_drive::Sides { count: u32::try_from(args.get("sides").and_then(serde_json::Value::as_u64).expect("the sides")).expect("the sides") };
    let made = session.prism(qymcad_drive::Prism { radius, height, sides }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn sweep(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let section = profile_of(args.get("section").expect("the section"));
    let path = path_of(args.get("path").expect("the path"));
    let made = session.sweep(qymcad_drive::Sweep { section, path }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn profile_of(section: &serde_json::Value) -> qymcad_drive::Profile {
    qymcad_drive::Profile {
        sketch: qymcad_drive::SketchId { id: section.get("sketch").and_then(serde_json::Value::as_u64).expect("the sketch") },
        contour: qymcad_drive::ContourId { id: section.get("contour").and_then(serde_json::Value::as_u64).expect("the contour") },
    }
}

fn path_of(path: &serde_json::Value) -> qymcad_drive::Path {
    qymcad_drive::Path {
        sketch: qymcad_drive::SketchId { id: path.get("sketch").and_then(serde_json::Value::as_u64).expect("the sketch") },
        contour: qymcad_drive::ContourId { id: path.get("contour").and_then(serde_json::Value::as_u64).expect("the contour") },
    }
}

fn loft(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let lower = qymcad_drive::Lower { profile: profile_of(args.get("lower").expect("the lower")) };
    let upper = qymcad_drive::Upper { profile: profile_of(args.get("upper").expect("the upper")) };
    let made = session.loft(qymcad_drive::Loft { lower, upper }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn push(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let face = qymcad_drive::FaceId { id: u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face") };
    let by = qymcad_drive::Shift { mm: args.get("distance").and_then(serde_json::Value::as_f64).expect("the distance") };
    let made = session.push(qymcad_drive::Push { body, face, by }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn remove_face(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let face = qymcad_drive::FaceId { id: u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face") };
    let made = session.remove_face(qymcad_drive::RemoveFace { body, face }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn replace_face(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let face = qymcad_drive::FaceId { id: u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face") };
    let surface = qymcad_drive::BodyId { id: args.get("surface").and_then(serde_json::Value::as_u64).expect("the surface") };
    let made = session.replace_face(qymcad_drive::ReplaceFace { body, face, surface }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn revolve(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let sketch = qymcad_drive::SketchId { id: args.get("sketch").and_then(serde_json::Value::as_u64).expect("the sketch") };
    let contour = qymcad_drive::ContourId { id: args.get("contour").and_then(serde_json::Value::as_u64).expect("the contour") };
    let axis = axis_of(args.get("axis").and_then(serde_json::Value::as_str).unwrap_or(""));
    let angle = qymcad_drive::Angle { deg: args.get("angle").and_then(serde_json::Value::as_f64).expect("the angle") };
    let made = session.revolve(qymcad_drive::Profile { sketch, contour }, axis, angle).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn bind(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = args.get("body").and_then(serde_json::Value::as_u64).expect("the body");
    let dimension = dimension_of(args.get("dimension").and_then(serde_json::Value::as_str).unwrap_or(""));
    let expression = args.get("expression").and_then(serde_json::Value::as_str).expect("the expression");
    session.bind(qymcad_drive::BodyId { id: body }, dimension, qymcad_drive::Expression { text: expression.to_string() }).map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn fillet(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = args.get("body").and_then(serde_json::Value::as_u64).expect("the body");
    let edge = u32::try_from(args.get("edge").and_then(serde_json::Value::as_u64).expect("the edge")).expect("the edge");
    let radius = args.get("radius").and_then(serde_json::Value::as_f64).expect("the radius");
    let made = session.fillet(qymcad_drive::BodyId { id: body }, qymcad_drive::EdgeId { id: edge }, qymcad_drive::Radius { mm: radius }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn chamfer(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = args.get("body").and_then(serde_json::Value::as_u64).expect("the body");
    let edge = u32::try_from(args.get("edge").and_then(serde_json::Value::as_u64).expect("the edge")).expect("the edge");
    let distance = args.get("distance").and_then(serde_json::Value::as_f64).expect("the distance");
    let made = session.chamfer(qymcad_drive::BodyId { id: body }, qymcad_drive::EdgeId { id: edge }, qymcad_drive::Distance { mm: distance }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn shell(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = args.get("body").and_then(serde_json::Value::as_u64).expect("the body");
    let face = u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face");
    let thickness = args.get("thickness").and_then(serde_json::Value::as_f64).expect("the thickness");
    let made = session.shell(qymcad_drive::BodyId { id: body }, qymcad_drive::FaceId { id: face }, qymcad_drive::Thickness { mm: thickness }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn hole(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = args.get("body").and_then(serde_json::Value::as_u64).expect("the body");
    let face = u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face");
    let diameter = args.get("diameter").and_then(serde_json::Value::as_f64).expect("the diameter");
    let depth = args.get("depth").and_then(serde_json::Value::as_f64).expect("the depth");
    let made =
        session.hole(qymcad_drive::BodyId { id: body }, qymcad_drive::FaceId { id: face }, qymcad_drive::Diameter { mm: diameter }, qymcad_drive::Depth { mm: depth }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn back(session: &mut qymcad_drive::Session) -> Result<serde_json::Value, String> {
    session.back().map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn parameter(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let name = args.get("name").and_then(serde_json::Value::as_str).expect("the name");
    let expression = args.get("expression").and_then(serde_json::Value::as_str).expect("the expression");
    session.parameter(qymcad_drive::ParameterName { text: name.to_string() }, qymcad_drive::Expression { text: expression.to_string() }).map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn part(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let place = args.get("place").and_then(serde_json::Value::as_array).expect("the place");
    let made = session.part(qymcad_drive::Part { place: qymcad_drive::Place { mat: matrix_of(place) } }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "part": made.id }))
}

fn mate(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let stay = qymcad_drive::PartId { id: args.get("stay").and_then(serde_json::Value::as_u64).expect("the stay") };
    let bring = qymcad_drive::PartId { id: args.get("bring").and_then(serde_json::Value::as_u64).expect("the bring") };
    let made = session.mate(qymcad_drive::Mate { stay: qymcad_drive::Stay { part: stay }, bring: qymcad_drive::Bring { part: bring } }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "mate": made.id }))
}

fn slide(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let stay = qymcad_drive::PartId { id: args.get("stay").and_then(serde_json::Value::as_u64).expect("the stay") };
    let bring = qymcad_drive::PartId { id: args.get("bring").and_then(serde_json::Value::as_u64).expect("the bring") };
    let by = qymcad_drive::Travel { mm: args.get("by").and_then(serde_json::Value::as_f64).expect("the by") };
    let made = session.slide(qymcad_drive::Slide { stay: qymcad_drive::Stay { part: stay }, bring: qymcad_drive::Bring { part: bring }, by }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "mate": made.id }))
}

fn auger(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let rim = qymcad_drive::EdgeId { id: u32::try_from(args.get("rim").and_then(serde_json::Value::as_u64).expect("the rim")).expect("the rim") };
    let outer = qymcad_drive::Outer { mm: args.get("outer").and_then(serde_json::Value::as_f64).expect("the outer") };
    let pitch = qymcad_drive::AugerPitch { mm: args.get("pitch").and_then(serde_json::Value::as_f64).expect("the pitch") };
    let thickness = qymcad_drive::Flight { mm: args.get("thickness").and_then(serde_json::Value::as_f64).expect("the thickness") };
    let length = qymcad_drive::AugerLength { mm: args.get("length").and_then(serde_json::Value::as_f64).expect("the length") };
    let made = session.auger(qymcad_drive::Auger { body, rim, outer, pitch, thickness, length }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn read(session: &mut qymcad_drive::Session) -> serde_json::Value {
    let doc = session.document();
    let bodies: Vec<serde_json::Value> = doc.bodies.iter().map(body_of).collect();
    let components: Vec<serde_json::Value> = doc.components.iter().map(component_of).collect();
    let mates: Vec<serde_json::Value> = doc.mates.iter().map(mate_of).collect();
    let sketches: Vec<serde_json::Value> = doc.sketches.iter().map(sketch_of).collect();
    let nodes: Vec<serde_json::Value> = doc.nodes.iter().map(node_of).collect();
    serde_json::json!({
        "rebuild": rebuild_of(&doc.rebuild),
        "refusals": refusals_of(&doc.rebuild),
        "components": components,
        "mates": mates,
        "sketches": sketches,
        "nodes": nodes,
        "bodies": bodies,
    })
}

include!("../ask_to_open.rs");

fn show(args: &serde_json::Value) -> Result<serde_json::Value, String> {
    ask_to_open(args.get("path").and_then(serde_json::Value::as_str).unwrap_or(""))?;
    Ok(serde_json::json!({}))
}

fn save(session: &qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = args.get("path").and_then(serde_json::Value::as_str).expect("the path");
    session.save(path).map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn open_document(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = args.get("path").and_then(serde_json::Value::as_str).expect("the path");
    *session = qymcad_drive::Session::open(path).map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn open_step(args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = args.get("path").and_then(serde_json::Value::as_str).expect("the path");
    let solids = qymcad_drive::Session::open_step(path).map_err(|err| err.text)?;
    let published: Vec<serde_json::Value> = solids.iter().map(solid_of).collect();
    Ok(serde_json::json!({ "solids": published }))
}

fn open_stl(args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = args.get("path").and_then(serde_json::Value::as_str).expect("the path");
    let read = qymcad_drive::Session::open_stl(path).map_err(|err| err.text)?;
    Ok(serde_json::json!({
        "triangles": read.triangles,
        "volume": read.volume.mm3,
        "low": corner_low(read.low),
        "high": corner_high(read.high),
    }))
}

fn solid_of(solid: &qymcad_drive::SolidRead) -> serde_json::Value {
    let (volume, count) = match solid.measure {
        qymcad_drive::Measure::Live { volume, solids } => (serde_json::json!(volume.mm3), serde_json::json!(solids)),
        qymcad_drive::Measure::Absent => (serde_json::Value::Null, serde_json::Value::Null),
    };
    serde_json::json!({
        "sound": sound_of(solid.sound),
        "solids": count,
        "volume": volume,
        "low": corner_low(solid.low),
        "high": corner_high(solid.high),
    })
}

fn corner_low(corner: qymcad_drive::LowCorner) -> serde_json::Value {
    serde_json::json!({ "x": corner.x.mm, "y": corner.y.mm, "z": corner.z.mm })
}

fn corner_high(corner: qymcad_drive::HighCorner) -> serde_json::Value {
    serde_json::json!({ "x": corner.x.mm, "y": corner.y.mm, "z": corner.z.mm })
}

fn save_step(session: &qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = args.get("path").and_then(serde_json::Value::as_str).expect("the path");
    // The kernel writes its transfer summary on standard output. Park that stream on standard error
    // for this call, then put it back, so the summary is not a protocol line.
    let parked = park_stdout();
    let wrote = session.save_step(path);
    drop(parked);
    wrote.map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

fn save_stl(session: &qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = args.get("path").and_then(serde_json::Value::as_str).expect("the path");
    session.save_stl(path).map_err(|err| err.text)?;
    Ok(serde_json::json!({}))
}

extern "C" {
    fn dup(fd: i32) -> i32;
    fn dup2(old_fd: i32, new_fd: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn fflush(stream: *mut std::ffi::c_void) -> i32;
    static mut stdout: *mut std::ffi::c_void;
}

struct Parked {
    saved: i32,
}

impl Drop for Parked {
    fn drop(&mut self) {
        unsafe {
            fflush(stdout);
            dup2(self.saved, 1);
            close(self.saved);
        }
    }
}

fn park_stdout() -> Parked {
    let saved = unsafe { dup(1) };
    if saved < 0 {
        panic!("the reply");
    }
    let moved = unsafe { dup2(2, 1) };
    if moved != 1 {
        panic!("the reply");
    }
    Parked { saved }
}

fn plane_of(name: &str) -> qymcad_drive::WorldPlane {
    match name {
        "xy" => qymcad_drive::WorldPlane::Xy,
        "xz" => qymcad_drive::WorldPlane::Xz,
        "yz" => qymcad_drive::WorldPlane::Yz,
        _ => panic!("the plane"),
    }
}

fn table_of(name: &str) -> qymcad_drive::WorldPlane {
    match name {
        "xy" => qymcad_drive::WorldPlane::Xy,
        _ => panic!("the plane"),
    }
}

fn split(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let plane = table_of(args.get("plane").and_then(serde_json::Value::as_str).expect("the plane"));
    let offset = qymcad_drive::Offset { mm: args.get("offset").and_then(serde_json::Value::as_f64).expect("the offset") };
    let made = session.split(qymcad_drive::Split { body, plane, offset }).map_err(|err| err.text)?;
    if made.bodies.len() != 2 {
        panic!("the split");
    }
    Ok(serde_json::json!({ "lower": made.bodies[0].id, "upper": made.bodies[1].id }))
}

fn split_faces(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let plane = table_of(args.get("plane").and_then(serde_json::Value::as_str).expect("the plane"));
    let offset = qymcad_drive::Offset { mm: args.get("offset").and_then(serde_json::Value::as_f64).expect("the offset") };
    let made = session.split_faces(qymcad_drive::Split { body, plane, offset }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn stitch(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let listed = args.get("sheets").and_then(serde_json::Value::as_array).expect("the sheets");
    let mut sheets = Vec::new();
    for entry in listed {
        sheets.push(qymcad_drive::BodyId { id: entry.as_u64().expect("the sheet") });
    }
    let gap = qymcad_drive::Gap { mm: args.get("gap").and_then(serde_json::Value::as_f64).expect("the gap") };
    let made = session.stitch(qymcad_drive::Stitch { sheets, gap }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn draft(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let tilted = qymcad_drive::Tilted { face: qymcad_drive::FaceId { id: u32::try_from(args.get("wall").and_then(serde_json::Value::as_u64).expect("the wall")).expect("the wall") } };
    let neutral = qymcad_drive::Neutral { face: qymcad_drive::FaceId { id: u32::try_from(args.get("neutral").and_then(serde_json::Value::as_u64).expect("the neutral")).expect("the neutral") } };
    let angle = qymcad_drive::Angle { deg: args.get("angle").and_then(serde_json::Value::as_f64).expect("the angle") };
    let made = session.draft(qymcad_drive::Draft { body, tilted, neutral, angle }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn circular(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let copies = qymcad_drive::Copies { n: u32::try_from(args.get("copies").and_then(serde_json::Value::as_u64).expect("the copies")).expect("the copies") };
    let turn = qymcad_drive::Angle { deg: args.get("angle").and_then(serde_json::Value::as_f64).expect("the angle") };
    let made = session.circular(qymcad_drive::Circular { body, copies, turn }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn linear(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let step = step_of(args.get("step").expect("the step"));
    let copies = qymcad_drive::Copies { n: u32::try_from(args.get("copies").and_then(serde_json::Value::as_u64).expect("the copies")).expect("the copies") };
    let made = session.linear(qymcad_drive::Linear { body, step, copies }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn step_of(step: &serde_json::Value) -> qymcad_drive::Step {
    qymcad_drive::Step {
        x: qymcad_drive::AlongX { mm: step.get("x").and_then(serde_json::Value::as_f64).expect("the step") },
        y: qymcad_drive::AlongY { mm: step.get("y").and_then(serde_json::Value::as_f64).expect("the step") },
        z: qymcad_drive::AlongZ { mm: step.get("z").and_then(serde_json::Value::as_f64).expect("the step") },
    }
}

fn mirror(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let plane = table_of(args.get("plane").and_then(serde_json::Value::as_str).expect("the plane"));
    let made = session.mirror(qymcad_drive::Mirror { body, plane }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn move_body(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let by = args.get("by").and_then(serde_json::Value::as_array).expect("the by");
    let made = session.move_body(qymcad_drive::Move { body, by: qymcad_drive::Place { mat: matrix_of(by) } }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn matrix_of(by: &[serde_json::Value]) -> [f64; 12] {
    let entries: &[serde_json::Value; 12] = by.try_into().unwrap_or_else(|_| panic!("the place"));
    std::array::from_fn(|index| entries[index].as_f64().expect("the place"))
}

fn offset_face(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let face = qymcad_drive::FaceId { id: u32::try_from(args.get("face").and_then(serde_json::Value::as_u64).expect("the face")).expect("the face") };
    let by = qymcad_drive::Outset { mm: args.get("distance").and_then(serde_json::Value::as_f64).expect("the distance") };
    let made = session.offset_face(qymcad_drive::OffsetFace { body, face, by }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn patch(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let body = qymcad_drive::BodyId { id: args.get("body").and_then(serde_json::Value::as_u64).expect("the body") };
    let edge = qymcad_drive::EdgeId { id: u32::try_from(args.get("edge").and_then(serde_json::Value::as_u64).expect("the edge")).expect("the edge") };
    let meet = meet_of(args.get("meet").and_then(serde_json::Value::as_str).expect("the meet"));
    let made = session.patch(qymcad_drive::Patch { body, edge, meet }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn meet_of(name: &str) -> qymcad_drive::Meet {
    match name {
        "position" => qymcad_drive::Meet::ByPosition,
        _ => panic!("the meet"),
    }
}

fn boolean(session: &mut qymcad_drive::Session, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let base = qymcad_drive::Base { body: qymcad_drive::BodyId { id: args.get("base").and_then(serde_json::Value::as_u64).expect("the base") } };
    let op = op_of(args.get("op").and_then(serde_json::Value::as_str).expect("the operation"));
    let tool = qymcad_drive::Tool { body: qymcad_drive::BodyId { id: args.get("tool").and_then(serde_json::Value::as_u64).expect("the tool") } };
    let made = session.boolean(qymcad_drive::Boolean { base, op, tool }).map_err(|err| err.text)?;
    Ok(serde_json::json!({ "body": made.id }))
}

fn op_of(word: &str) -> qymcad_drive::BoolOp {
    match word {
        "cut" => qymcad_drive::BoolOp::Cut,
        _ => panic!("the operation"),
    }
}

fn axis_of(name: &str) -> qymcad_drive::SketchAxis {
    match name {
        "x" => qymcad_drive::SketchAxis::X,
        _ => panic!("the axis"),
    }
}

fn dimension_of(name: &str) -> qymcad_drive::Dimension {
    match name {
        "height" => qymcad_drive::Dimension::Height,
        _ => panic!("the dimension"),
    }
}

fn corner_of(args: &serde_json::Value, key: &str) -> (f64, f64) {
    let value = args.get(key).unwrap_or(&serde_json::Value::Null);
    let x = value.get("x").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
    let y = value.get("y").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
    (x, y)
}

fn mate_of(mate: &qymcad_drive::Fastened) -> serde_json::Value {
    serde_json::json!({
        "id": mate.id,
        "name": mate.name,
        "stay": mate.stay.part.id,
        "bring": mate.bring.part.id,
    })
}

fn component_of(component: &qymcad_drive::Component) -> serde_json::Value {
    serde_json::json!({
        "id": component.id,
        "name": component.name,
        "kind": kind_of(component.kind),
        "parent": id_or_null(component.parent),
        "place": component.place.mat,
    })
}

fn sketch_of(sketch: &qymcad_drive::Sketch) -> serde_json::Value {
    let contours: Vec<serde_json::Value> = sketch.contours.iter().map(contour_of).collect();
    serde_json::json!({
        "id": sketch.id,
        "name": sketch.name,
        "plane": plane_name(&sketch.plane),
        "contours": contours,
    })
}

fn contour_of(contour: &qymcad_drive::Contour) -> serde_json::Value {
    serde_json::json!({ "id": contour.id, "form": form_of(contour.form) })
}

fn node_of(node: &qymcad_drive::Node) -> serde_json::Value {
    serde_json::json!({ "id": node.id, "name": node.name, "parent": id_or_null(node.parent) })
}

fn id_or_null(id: Option<u64>) -> serde_json::Value {
    match id {
        Some(id) => serde_json::Value::from(id),
        None => serde_json::Value::Null,
    }
}

fn kind_of(kind: qymcad_drive::Kind) -> &'static str {
    match kind {
        qymcad_drive::Kind::Part => "part",
        qymcad_drive::Kind::Assembly => "assembly",
    }
}

fn form_of(form: qymcad_drive::Form) -> &'static str {
    match form {
        qymcad_drive::Form::Closed => "closed",
        qymcad_drive::Form::Open => "open",
    }
}

fn plane_name(plane: &qymcad_drive::Plane) -> serde_json::Value {
    match plane {
        qymcad_drive::Plane::Xy => serde_json::json!("xy"),
        qymcad_drive::Plane::Xz => serde_json::json!("xz"),
        qymcad_drive::Plane::Yz => serde_json::json!("yz"),
        qymcad_drive::Plane::Datum { id } => serde_json::json!({ "datum": id }),
        qymcad_drive::Plane::Face { body } => serde_json::json!({ "body": body }),
    }
}

fn body_of(body: &qymcad_drive::Body) -> serde_json::Value {
    let faces: Vec<serde_json::Value> = body.faces.iter().map(face_of).collect();
    let edges: Vec<serde_json::Value> = body.edges.iter().map(edge_of).collect();
    match body.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            serde_json::json!({ "id": body.id, "sound": sound_of(body.sound), "solids": solids, "volume": volume.mm3, "mesh": body.mesh.mm3, "faces": faces, "edges": edges })
        }
        qymcad_drive::Measure::Absent => {
            serde_json::json!({ "id": body.id, "sound": sound_of(body.sound), "solids": serde_json::Value::Null, "volume": serde_json::Value::Null, "mesh": body.mesh.mm3, "faces": faces, "edges": edges })
        }
    }
}

fn face_of(face: &qymcad_drive::Face) -> serde_json::Value {
    match face.role {
        qymcad_drive::FaceRole::CapStart => serde_json::json!({ "id": face.id.id, "role": "cap-start", "area": face.area.mm2 }),
        qymcad_drive::FaceRole::CapEnd => serde_json::json!({ "id": face.id.id, "role": "cap-end", "area": face.area.mm2 }),
        qymcad_drive::FaceRole::Other => serde_json::json!({ "id": face.id.id, "role": "other", "area": face.area.mm2 }),
        qymcad_drive::FaceRole::Wall { source } => serde_json::json!({ "id": face.id.id, "role": "wall", "source": source.id, "area": face.area.mm2 }),
    }
}

fn edge_of(edge: &qymcad_drive::Edge) -> serde_json::Value {
    serde_json::json!({ "id": edge.id.id, "length": edge.length.mm })
}

fn sound_of(sound: qymcad_drive::Sound) -> &'static str {
    match sound {
        qymcad_drive::Sound::Valid => "valid",
        qymcad_drive::Sound::Invalid => "invalid",
    }
}

fn rebuild_of(rebuild: &qymcad_drive::Rebuild) -> &'static str {
    match rebuild {
        qymcad_drive::Rebuild::Quiet => "quiet",
        qymcad_drive::Rebuild::Refused { .. } => "refused",
        qymcad_drive::Rebuild::Stopped => "stopped",
    }
}

fn refusals_of(rebuild: &qymcad_drive::Rebuild) -> Vec<serde_json::Value> {
    match rebuild {
        qymcad_drive::Rebuild::Refused { refusals } => refusals.iter().map(|refusal| serde_json::json!({ "node": refusal.node, "text": refusal.text })).collect(),
        qymcad_drive::Rebuild::Quiet | qymcad_drive::Rebuild::Stopped => Vec::new(),
    }
}

fn send(out: &mut impl Write, msg: &serde_json::Value, result: serde_json::Value) {
    let reply = serde_json::json!({
        "jsonrpc": "2.0",
        "id": msg.get("id").cloned().unwrap_or(serde_json::Value::Null),
        "result": result,
    });
    writeln!(out, "{reply}").expect("the reply");
    out.flush().expect("the reply");
}
