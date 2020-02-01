# Hooker
A Rust program to run Linux commands when a HTTP request is received at a configurable endpoint.

## How To Use
```bash
./hooker [ip address] [port] [config path]
```

* `ip address` - IP Address to bind to
* `port` - Port to listen on
* `config path` - Path to a directory to look for config files

## Configuration
Configuring end points is done via JSON files. The config path argument takes a path to a directory. Endpoints are then
configured by creating JSON files in the configuration directory with the following format:

```json
{
  "command": "touch /home/user test",
  "end_point": "coolpoint123"
}
```

* `command` - Linux command to run when a HTTP request is received
* `end_point` - Endpoint to listen on. The above example would point to `http://[ip_address]:[port]/coolpoint123`