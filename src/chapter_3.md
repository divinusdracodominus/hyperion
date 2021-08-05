# Server Configurations

### Example Configuration file
```toml
root = "/cardinal/hyperion/html"
log = "verbose"
listen = "0.0.0.0:8080"
blacklist = ["/cardinal/hyperion/html/.git"]

[[controllers]]
resource = "/cardinal/hyperion/html/loggedin.html"
controller = "/cardinal/hyperion/html/controller.ion"
```

### blacklist

the blacklist flags serves as a way to denote files or directories that shouldn't be indexed by hyperion, and thus shouldn't be served to clients. 

### whitelist

whitelist is the opposite of blacklist, and sets a list of files and directories that are the only things served to the client.

WARNING: it is possible to whitelist html/ thereby ignoring all blacklists, this is not advisable

### listen
defines the socket addrs, that is the ip addrs:port that the server should listen on. NOTE: if you try and listen on anything lower then 1024, ensurue the process running this server has higher privileges then std user, as those ports are restricted.

### log

log defines a level of detail that should be used when writing to log files, the current values are silent, and verbose, owing to the fact that a well defined log format hasn't been decided on, the current implementation is either all or nothing, this will change in future versions.

### controllers

the \[\[controllers\]\] attribute defines a redirect from one file or directory, to a given script that checks for example, @SESSION["active"] to see if the client has the right to access the given resource. 