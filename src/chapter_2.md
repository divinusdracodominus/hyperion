# Builtin Variables

the variables outlined in this chapter are stored as [HashMaps](https://doc.rust-lang.org/std/collections/struct.HashMap.html) on the shell's stack. Thse HashMaps map a key to a value, and can be accessed using the "@" symbol. for example

```
let operation = @GET["operation"]
let username = @POST["username"]
```

## HTTP Request Parameters

### COOKIE Variable

used to retrieve cookie values stored in HTTP headers that get passed back to the server from the client. 

### SESSION Variable

a session is stored by ID as a cookie that the client stores, which is a hex string, that when returned to the server is used to map a key, a session variable name, to the session variable's value, this is done using a reader writter lock stored globally and used to persist variables accross requests, and pages.

```sh
# if session_start isn't called, 
# there isn't a garuntee that there will be a map that stores session variables, 
# and thus setting session variables may fail.
session_start
set_session_variable "active" "true"
let active = @SESSION["active"]
```

### SERVER Variable

the SERVER variable provides information on the server, and the client that initiated the request. see [w3schools](https://www.w3schools.com/php/php_superglobals_server.asp) for more information. 
