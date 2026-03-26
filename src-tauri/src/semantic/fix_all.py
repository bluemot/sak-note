import re

# Fix parser.rs line 332
with open('parser.rs', 'r') as f:
    content = f.read()

# Fix the import regex capture
content = content.replace(
    'let module = cap.get(1).map(|m| m.as_str()).unwrap_or("")',
    'let module = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();'
)

with open('parser.rs', 'w') as f:
    f.write(content)

print("Fixed parser.rs")

# Fix bridge.rs line 254
with open('bridge.rs', 'r') as f:
    content = f.read()

content = content.replace(
    r"r\"import\s+(.+)\s+from\s+['\"]?(.+?)['\"]?\"",
    r"r#\"import\s+(.+)\s+from\s+['\"\'](.+)['\"\']\"#"
)

with open('bridge.rs', 'w') as f:
    f.write(content)

print("Fixed bridge.rs")
