with open('src/layer1/bureaucracy_of_scarcity.rs', 'r') as f:
    code = f.read()

code = code.replace("JobBoard { available_bureaucrat_jobs: vec![] }", "JobBoard { available_bureaucrat_jobs: 0 }")

with open('src/layer1/bureaucracy_of_scarcity.rs', 'w') as f:
    f.write(code)
