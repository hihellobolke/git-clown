# Project Goals
The goal of this project is to create a rust based binary called git-clown. It used is to clone a git repo, and based on the url, it creates directory structure under the directory $WORKSPACE_ROOT. 

For example if the urls was https://github.com/example/repo.git, then 

- Step 1 - creating directory: create the directory `$WORKSPACE_ROOT/github.com/example`
- Step 2 - finding .envrc or .env: check if there's is an .envrc or .env file in the directory example, if not it will check it's parent directory, and so on until it finds the .envrc or .env file or reaches $WORKSPACE_ROOT
- Step 3 - sourcing env: whatever env file is found, you load it using dotenvy, and execute the `git clone` 
and show the stderr and stdout of the `git clone` command and exist with the same exit code as the `git clone` command.

