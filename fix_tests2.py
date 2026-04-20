with open('src/ui/selection.rs', 'r') as f:
    text = f.read()

text = text.replace('    fn test_observer_awareness_adds_component() {', '    #[cfg(feature = "nova")]\n    fn test_observer_awareness_adds_component() {\n        use crate::layer1::observer::Observed;\n        use crate::ui::selection::observer_awareness_system;')
text = text.replace('    fn test_observer_awareness_removes_component() {', '    #[cfg(feature = "nova")]\n    fn test_observer_awareness_removes_component() {\n        use crate::layer1::observer::Observed;\n        use crate::ui::selection::observer_awareness_system;')

with open('src/ui/selection.rs', 'w') as f:
    f.write(text)
