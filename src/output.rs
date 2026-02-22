use serde_json::{json, Value};

pub fn emit_ok(json_mode: bool, replay_id: Option<&str>, extra: Value) {
    if json_mode {
        let mut obj = json!({
            "tool": "musketeer",
            "version": "1",
            "status": "ok",
            "replay_id": replay_id,
            "errors": []
        });
        if let Some(map) = extra.as_object() {
            let base = obj.as_object_mut().expect("object");
            for (k, v) in map {
                base.insert(k.clone(), v.clone());
            }
        }
        println!("{}", serde_json::to_string(&obj).expect("json"));
    }
}

pub fn emit_err(json_mode: bool, replay_id: Option<&str>, code: &str, message: &str) {
    if json_mode {
        let obj = json!({
            "tool": "musketeer",
            "version": "1",
            "status": "error",
            "replay_id": replay_id,
            "errors": [code, message]
        });
        println!("{}", serde_json::to_string(&obj).expect("json"));
    } else {
        eprintln!("{message}");
    }
}
