import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''    app.add_event::<crate::layer1::gravitational_debt::DebtReleaseEvent>();''',
'''    app.add_event::<crate::layer1::gravitational_debt::DebtReleaseEvent>();'''
)

with open('src/simulation.rs', 'w') as f:
    f.write(content)
