## DFACE(Rusty)

DFACE is a lightweight CLI-based webpage defacement detection and monitoring utility. It uses the Mozilla Geckodriver for page access, and SQLite for object storage. 
Defacements are detected through a cumulative similarity from last shot value. This value is generated through the analysis of both Fuzzy hashing(ssdeep), as well as 
perceptual hashes.

## Table of Contents
- Features
- Installation
- Usage  
- Configuration    
- Contributing
- License
    

## Features

- ✅ SSDeep and Phash
- ✅ Historical page comparison
- ✅ Verbose operation
- ✅ Entirely CLI accessible (No DB Browser needed for log analysis, new monitor creation, etc.)

- Lightweight database: This program depends on an SQLITE database, and upon the creation of the file will create 3 tables. These tables are as follows:

- ## Monitors:
- This holds important values pertaining to the URL's you wish to monitor. Here you will find a URL(Primary Key), a threshold(Indication of defacement in the form of a similarity value 1-100), a frequency value(Number of seconds between each run over the specific URL, and a retention value that determines the amount of time you wish to keep logs for this URL.
- ## Pages:
- In pages, you will find a timestamp, a URL, the specified URL's source code, a full page screenshot of the page instance at which it was last visited by the tool, as well as both hash values taken by the tool.
- ## LOGS:
- In the logs table, you will find a timestamp, a log type(Either log, or alert), as well as a message. 


## Installation

To install the Rusty version of DFACE, simply install the executeable from this repo. If you're doing a source code install, please ensure you have all dependencies
    noted in cargo.toml installed. Ensure you have sqlite installed on your computer, as well as Mozillas Geckodriver. Currently compiled in Windows binary only. Linux
    Binary coming soon. If you wish to configure your DFACE instance for remote logging, this is left up to the end user. Therefore, the installation of the source code is necessary in this case.


## Usage

Thanks to the simple nature of this applcation, the usage is simple itself. Ensure you're in a CMD or Powershell instance in DFACE's working directory, and enter the
    DFACE command. To view all available arguments, enter DFACE --help. Please ensure that you read the info lines for the arguments in detail, as some arguments require the
    invokation of others.


## Configuration:

If installing the compiled executable, minimal configuration is needed. Once again, all that is needed as a dependency for the executables function would be sqlite,
    and geckodriver. If installing the source code, please ensure all dependencies listed in the cargo.toml are installed in your environment. At this time, syslog messages and other logging mediums are not natively supported. To configure DFACE for remote logging to a syslog server, or other mediums, a source code installation is necessary, for the end user to edit the code accordingly.


## Contributing

The DFACE Repo is currently locked from contribution


## License

Licensed under the MIT license(Placeholder)



