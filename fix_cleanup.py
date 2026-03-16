import re

with open('src/layer1/systems/cleanup.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''            update_event_buffer::<crate::layer1::social::grievances::NewGrievanceEvent>,
            update_event_buffer::<crate::layer1::social::grievances::ResolveGrievanceEvent>,''',
'''            update_event_buffer::<crate::layer1::social::grievances::NewGrievanceEvent>,
            update_event_buffer::<crate::layer1::social::grievances::ResolveGrievanceEvent>,
            update_event_buffer::<crate::layer1::gravitational_debt::DebtReleaseEvent>,''')

with open('src/layer1/systems/cleanup.rs', 'w') as f:
    f.write(content)
