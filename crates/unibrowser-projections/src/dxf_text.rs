/// Extract TEXT/MTEXT string values and layer names from DXF ASCII.
pub fn dxf_text_and_layers(data: &str) -> (String, Vec<String>) {
    let lines: Vec<&str> = data.lines().map(str::trim).collect();
    let mut layers = Vec::new();
    let mut text_parts = Vec::new();
    let mut entity: Option<&str> = None;
    let mut i = 0;
    while i + 1 < lines.len() {
        let code = lines[i];
        let value = lines[i + 1];
        if code == "0" {
            entity = Some(value);
        } else if code == "8" && !value.is_empty() && !layers.contains(&value.to_string()) {
            layers.push(value.to_string());
        } else if code == "1" {
            if matches!(entity, Some("TEXT") | Some("MTEXT")) && !value.is_empty() {
                text_parts.push(value.to_string());
            }
        }
        i += 2;
    }
    (text_parts.join("\n"), layers)
}
