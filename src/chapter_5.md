# Planned Features

## Proper configurability

I plan to implement a means of controlling, not just through command line, but also through config files, information such as what address and port to listen on, what files are executable, what files aren't, what is servable, and what isn't, as well as what requests that serve will handle, and what requests it won't.

## Version Perfect Environment Setup

the use and distribution of a cargo like build tool, and package manager for the framework, that will allow not only minimal binary program garuntees, by operating in a manner similar to nix, but will also support embedding custom builtins into the shell executing .ion files.

### Version Perfect Build System

The plan is to use git hashes to be able to recover or control exact code changes within codebases to allow compiled binaries to have the expected features. This is for the most part already implemented in another code base that will be merged into this base.

### Environment control
Either use docker, or wasmtime to control what programs are accessable by given scripts. for example su, and sudo may not be mapped into the container, and as a result, neither of these programs will be usable, this of course helps to mitigate any unforseen arbitrary execution vulnerabilities that may come along.

### Potential Refactors
have root point to a directory containing, config.toml, and html directory where the html directory is served.

a global config path.