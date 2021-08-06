# Web Socket Support

## Configuration

Web sockets can be configured to listen on various ports, by placing in the config file a list of addresses that should be used for the respective sockets, the file used to handle these requests is passed as the path in the Request

#### Example Configuration
```
root = "/cardinal/hyperion/html"
log = "verbose"
listen = "0.0.0.0:8080"
blacklist = ["/cardinal/hyperion/html/.git"]
socket = ["12.0.0.1:1337"]
```

## Structure

unlike standard http requests, whenever a websocket connection is established, instead of calling the same script over and over again, a callback is used to send data to the script.

## How Scripts Are Selected

Web sockets can have query strings, as a result, the ion file responsible for running a websocket connection can either be passed on the path relative to the project root, or can be defined.