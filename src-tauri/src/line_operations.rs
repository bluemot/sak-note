//! Line Operations for Editor
//!
//! Provides Notepad++ style line operations:
//! - Duplicate line (Ctrl+D)
//! - Move line up/down (Ctrl+Shift+Up/Down)
//! - Delete line (Ctrl+L)
//! - Join lines
//! - Split lines

use serde_json::Value;

/// Duplicate current line
#[tauri::command]
pub fn edit_duplicate_line(content: String, line_number: usize) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if line_number == 0 || line_number > lines.len() {
        return Err("Invalid line number".to_string());
    }

    let line_idx = line_number - 1;
    let mut result_lines = lines.clone();
    result_lines.insert(line_idx, lines[line_idx]);

    Ok(serde_json::json!({
        "content": result_lines.join("\n"),
        "new_line_number": line_number + 1
    }))
}

/// Move line up
#[tauri::command]
pub fn edit_move_line_up(content: String, line_number: usize) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if line_number <= 1 || line_number > lines.len() {
        return Err("Cannot move line up".to_string());
    }

    let line_idx = line_number - 1;
    let mut result_lines = lines.clone();
    result_lines.swap(line_idx, line_idx - 1);

    Ok(serde_json::json!({
        "content": result_lines.join("\n"),
        "new_line_number": line_number - 1
    }))
}

/// Move line down
#[tauri::command]
pub fn edit_move_line_down(content: String, line_number: usize) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if line_number == 0 || line_number >= lines.len() {
        return Err("Cannot move line down".to_string());
    }

    let line_idx = line_number - 1;
    let mut result_lines = lines.clone();
    result_lines.swap(line_idx, line_idx + 1);

    Ok(serde_json::json!({
        "content": result_lines.join("\n"),
        "new_line_number": line_number + 1
    }))
}

/// Delete line
#[tauri::command]
pub fn edit_delete_line(content: String, line_number: usize) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if line_number == 0 || line_number > lines.len() {
        return Err("Invalid line number".to_string());
    }

    let line_idx = line_number - 1;
    let mut result_lines: Vec<&str> = lines.clone();
    result_lines.remove(line_idx);

    let new_line = if result_lines.is_empty() {
        1
    } else if line_number > result_lines.len() {
        result_lines.len()
    } else {
        line_number
    };

    Ok(serde_json::json!({
        "content": result_lines.join("\n"),
        "new_line_number": new_line
    }))
}

/// Join lines (current with next)
#[tauri::command]
pub fn edit_join_lines(content: String, start_line: usize, end_line: usize) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if start_line == 0 || end_line > lines.len() || start_line > end_line {
        return Err("Invalid line range".to_string());
    }

    let start_idx = start_line - 1;
    let end_idx = end_line - 1;

    let before: Vec<&str> = lines[..start_idx].to_vec();
    let joined = lines[start_idx..=end_idx].join(" ");
    let after: Vec<&str> = lines[end_idx + 1..].to_vec();

    let mut result = before;
    result.push(&joined);
    result.extend(after);

    Ok(serde_json::json!({
        "content": result.join("\n"),
        "new_line_number": start_line
    }))
}

/// Split line at cursor
#[tauri::command]
pub fn edit_split_line(content: String, line_number: usize, column: usize) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if line_number == 0 || line_number > lines.len() {
        return Err("Invalid line number".to_string());
    }

    let line_idx = line_number - 1;
    let line = lines[line_idx];
    
    if column > line.len() {
        return Err("Invalid column".to_string());
    }

    let first_part = &line[..column];
    let second_part = &line[column..];

    let mut result_lines: Vec<&str> = lines[..line_idx].to_vec();
    result_lines.push(first_part);
    result_lines.push(second_part);
    result_lines.extend(&lines[line_idx + 1..]);

    Ok(serde_json::json!({
        "content": result_lines.join("\n"),
        "new_line_number": line_number + 1
    }))
}

/// Trim trailing whitespace
#[tauri::command]
pub fn edit_trim_trailing(content: String) -> Result<Value, String> {
    let trimmed: Vec<String> = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();

    Ok(serde_json::json!({
        "content": trimmed.join("\n"),
        "changes": trimmed.iter().enumerate().filter(|(i, l)| *l != content.lines().nth(*i).unwrap()).count()
    }))
}

/// Trim leading whitespace
#[tauri::command]
pub fn edit_trim_leading(content: String) -> Result<Value, String> {
    let trimmed: Vec<String> = content
        .lines()
        .map(|line| line.trim_start().to_string())
        .collect();

    Ok(serde_json::json!({
        "content": trimmed.join("\n"),
        "changes": trimmed.iter().enumerate().filter(|(i, l)| *l != content.lines().nth(*i).unwrap()).count()
    }))
}

/// Trim all whitespace
#[tauri::command]
pub fn edit_trim_all(content: String) -> Result<Value, String> {
    let trimmed: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .collect();

    Ok(serde_json::json!({
        "content": trimmed.join("\n"),
        "changes": trimmed.iter().enumerate().filter(|(i, l)| *l != content.lines().nth(*i).unwrap()).count()
    }))
}

/// Convert to uppercase
#[tauri::command]
pub fn edit_to_uppercase(content: String, start: usize, end: usize) -> Result<Value, String> {
    if end > content.len() {
        return Err("Invalid range".to_string());
    }

    let mut result = content.clone();
    let selected = &content[start..end];
    result.replace_range(start..end, &selected.to_uppercase());

    Ok(serde_json::json!({
        "content": result
    }))
}

/// Convert to lowercase
#[tauri::command]
pub fn edit_to_lowercase(content: String, start: usize, end: usize) -> Result<Value, String> {
    if end > content.len() {
        return Err("Invalid range".to_string());
    }

    let mut result = content.clone();
    let selected = &content[start..end];
    result.replace_range(start..end, &selected.to_lowercase());

    Ok(serde_json::json!({
        "content": result
    }))
}

/// Sort lines
#[tauri::command]
pub fn edit_sort_lines(
    content: String, 
    start_line: usize, 
    end_line: usize,
    ascending: bool
) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if start_line == 0 || end_line > lines.len() || start_line > end_line {
        return Err("Invalid line range".to_string());
    }

    let start_idx = start_line - 1;
    let end_idx = end_line - 1;

    let mut to_sort: Vec<&str> = lines[start_idx..=end_idx].to_vec();
    
    // Try numeric sort first, fall back to lexicographic
    let is_numeric = to_sort.iter().all(|l| l.trim().parse::<f64>().is_ok());
    
    if is_numeric {
        to_sort.sort_by(|a, b| {
            let a_num = a.trim().parse::<f64>().unwrap_or(0.0);
            let b_num = b.trim().parse::<f64>().unwrap_or(0.0);
            if ascending {
                a_num.partial_cmp(&b_num).unwrap()
            } else {
                b_num.partial_cmp(&a_num).unwrap()
            }
        });
    } else {
        if ascending {
            to_sort.sort();
        } else {
            to_sort.sort_by(|a, b| b.cmp(a));
        }
    }

    let mut result: Vec<&str> = lines[..start_idx].to_vec();
    result.extend(to_sort);
    result.extend(&lines[end_idx + 1..]);

    Ok(serde_json::json!({
        "content": result.join("\n"),
        "sort_type": if is_numeric { "numeric" } else { "lexicographic" },
        "order": if ascending { "ascending" } else { "descending" }
    }))
}

/// Toggle line comment
#[tauri::command]
pub fn edit_toggle_comment(
    content: String,
    start_line: usize,
    end_line: usize,
    comment_prefix: String
) -> Result<Value, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if start_line == 0 || end_line > lines.len() || start_line > end_line {
        return Err("Invalid line range".to_string());
    }

    let start_idx = start_line - 1;
    let end_idx = end_line - 1;

    // Check if all lines are already commented
    let all_commented = lines[start_idx..=end_idx]
        .iter()
        .all(|line| line.trim_start().starts_with(&comment_prefix));

    let mut result: Vec<String> = lines[..start_idx]
        .iter()
        .map(|s| s.to_string())
        .collect();

    for i in start_idx..=end_idx {
        let line = lines[i];
        if all_commented {
            // Uncomment
            let trimmed = line.trim_start();
            if let Some(_pos) = trimmed.find(&comment_prefix) {
                let spaces = line.len() - line.trim_start().len();
                let new_line = format!(
                    "{}{}",
                    &line[..spaces],
                    &trimmed[comment_prefix.len()..].trim_start()
                );
                result.push(new_line);
            } else {
                result.push(line.to_string());
            }
        } else {
            // Comment
            result.push(format!("{} {}", comment_prefix, line));
        }
    }

    result.extend(lines[end_idx + 1..].iter().map(|s| s.to_string()));

    Ok(serde_json::json!({
        "content": result.join("\n"),
        "action": if all_commented { "uncommented" } else { "commented" }
    }))
}
