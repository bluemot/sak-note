import re

with open('parser.rs', 'r') as f:
    lines = f.readlines()

# Replace line 209 (0-indexed: 208)
lines[208] = '        let import_regex = Regex::new(r#"^import\\s+(.+)\\s+from\\s+[\'"](.+)[\'"];?"#).unwrap();\n'

with open('parser.rs', 'w') as f:
    f.writelines(lines)

print("Fixed line 209")
