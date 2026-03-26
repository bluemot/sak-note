with open('bridge.rs', 'r') as f:
    lines = f.readlines()

# Line 254 is index 253
lines[253] = '''        let re = regex::Regex::new(r#"import\\s+(.+)\\s+from\\s+[\'"\'](.+)[\'"\']"#).unwrap();\n'''

with open('bridge.rs', 'w') as f:
    f.writelines(lines)

print("Fixed bridge.rs line 254")
