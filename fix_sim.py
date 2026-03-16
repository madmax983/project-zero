import re

with open('src/lib.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''    app.add_event::<GridOverloadEvent>();''',
'''    app.add_event::<GridOverloadEvent>();
    app.add_event::<crate::layer1::gravitational_debt::DebtReleaseEvent>();'''
)

with open('src/lib.rs', 'w') as f:
    f.write(content)
