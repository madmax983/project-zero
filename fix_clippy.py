with open('src/ui/inspector.rs', 'r') as f:
    content = f.read()

content = content.replace("let mut widgets: Vec<Box<dyn FnOnce(&mut Frame, Rect)>> = Vec::new();", """type InspectorWidget<'a> = Box<dyn FnOnce(&mut Frame, Rect) + 'a>;
    let mut widgets: Vec<InspectorWidget> = Vec::new();""")

with open('src/ui/inspector.rs', 'w') as f:
    f.write(content)
