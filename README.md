# A better alternative to JS and PHP for easy backend

## The core idea
the goal of this project is to make a backend stack that has less of the common vulnerabilities associated with traditional PHP applications, but also to provide a more diverse ecosystem then Node.js by leveraging a more natural web assembly backend, principally wasmtime.

## Origins
The project was originally intended to have a lightweight replacement to PHP through the use of the ion shell language, but it was determined that the ion shell language would not work well for this application, as it was far to easy to have arbitrary execution, thus the project shall be ported to use wasmtime rather then ion.

## Why wasmtime
The goal is security, and simplisity, the use of wasmtinme will allow for precise permission declaration, including what host system features can be accessed, such as networking, sql, files, etc, however unlike a virtual machine this system would have the added security htat comes with the architecting of the web assembly framework, and would also be more performant on non native CPUs then an x96 VM running on arm CPU for example. However owing to the fact that the programming will completely be running in userspace rather then kernel space, a compromised wasmtime instance shoudld it happen, will have far less fallout then a compromised docker container (similar to podman, however this also gives more configurability, programmability then a a container system)

