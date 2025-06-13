---
mode: 'agent'
tools: ['githubRepo', 'codebase']
description: 'Generate the code based on the copilot-instructions.md file.'
---
Your goal is to create write src/main.rs file based on the instructions in the copilot-instructions.md file.


### Command line parsing
* Use clap to parse the command line arguments.
* Execute git clone command in the directory that you created
* Use the provided workspace root from the environment variable `WORKSPACE_ROOT`, if not set default to the path ws directory under users homedirectory.

### Directory structure
* Use the provided git url to create the directory structure.
* if the directory structure exists and if the repo is already cloned, it should not clone again, but just print a message that the repo is already cloned.

### Env
* Use dotenvy to load the environment variables from the .envrc or .env file.
* Handle `export` command in the .envrc or .env file.
* If the .envrc or .env file is not found, it should check the parent directory and so on until it finds the .envrc or .env file or reaches $WORKSPACE_ROOT.

### async exec
* use tokio for async functionality.
* use tokio process for executing the git clone command.
* pass the environment variables to the git clone command.
* set the current working directory to the directory you created for git clone command.

### Logging:
* use logging to output to stdout and stderr. And use colored logging
* at every step, log the output, like:
    * when you successfully parse git url, 
    * log the path where git clone should be run, when you create the directory,
    * when you find the .envrc or .env file,
    * when you execute the git clone command

### Testing:
* write test cases for all the functionality you implement.

## Documentation:
* Use rust doc comments to document the code.
* Use cargo doc to generate the documentation.
* Use cargo fmt to format the code.
* Add a README.md file to the project with the following sections:
    * Introduction
    * Installation
    * Usage
    * Contributing
    * License
