## DFACE(Rusty)

DFACE is a lightweight CLI-based webpage defacement detection and monitoring utility. It uses the Mozilla Geckodriver for page access, and SQLite for object storage. 
Defacements are detected through a cumulative similarity from last shot value. This value is generated through the analysis of both Fuzzy hashing(ssdeep), as well as 
perceptual hashes.

## Table of Contents
- Features:
  - SSDeep and Phash,
  - Historical page comparison
  - Verbose operation
  - Entirely CLI accessible (No DB Browser needed for log analysis, new monitor creation, etc.)
  
- Installation:
    To install the Rusty version of DFACE, simply install the executeable from this repo. If you're doing a source code install, please ensure you have all dependencies
    noted in cargo.toml installed. Ensure you have sqlite installed on your computer, as well as Mozillas Geckodriver. Currently compiled in Windows binary only. Linux
    Binary coming soon.
- Usage:
    Thanks to the simple nature of this applcation, the usage is simple itself. Ensure you're in a CMD or Powershell instance in DFACE's working directory, and enter the
    DFACE command. To view all available arguments, enter DFACE --help. Please ensure that you read the info lines for the arguments in detail, as some arguments require the
    invokation of others.
- Configuration:
    If installing the compiled executable, minimal configuration is needed. Once again, all that is needed as a dependency for the executables function would be sqlite,
    and geckodriver. If installing the source code, please ensure all dependencies listed in the cargo.toml are installed in your environment
- Contributing:
      The DFACE Repo is currently unavailable for contirbution.
- License:
    Licensed under the MIT License(PLACEHOLDER)
    

## Features

- ✅ SSDeep and Phash
- ✅ Historical page comparison
- ✅ Verbose operation
- ✅ Entirely CLI accessible (No DB Browser needed for log analysis, new monitor creation, etc.)



