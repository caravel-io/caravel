# caravel

![caravel ship](https://github.com/caravel-io/caravel/blob/main/media/caravel.jpg?raw=true)

Next-gen configuration management using Rust and Lua

> [!WARNING]  
> 🚧 Caravel is under active development, and is in no way, shape, or form ready for use. 🚧
>
> 🚧 Please enjoy our empty promises in the Motivation section below! 🚧

## Motivation

While the technology landscape has changed and cloud-native infrastructure is popular, sometimes you've just got
a bunch of Linux servers and you need a sane way to manage their configurations. Traditionally, you'd use a tool
like Ansible, Puppet, or Chef. Those are great tools! If they work for you, then there's no reason to try Caravel.
However, if you're tired of the slow performance and YAML-hell that is Ansible, or the complicated infrastructure, setup,
and learning curve of Puppet, then Caravel might be worth a try.

The purpose of Caravel is to be super flexible and "blazingly fast". Using Lua as a programmable configuration language
and Rust as the engine allows the best of both worlds! 

- Caravel can be ran agentless or agentful. 

- You can direct connect or use a proxy. 

- You can build out an inventory or just point and shoot at any number of servers.

Caravel is and will always be open-source and free.

The name is based on a type of ship:

> The caravel is a small maneuverable sailing ship that was known for its agility and speed.

## Usage

TBD

## Writing a Manifest

Caravel applies resources based on a manifest that you write in Lua. 

A very simple "package-file-service" manifest might look like:

```lua
caravel.core.package({
    name = "nginx",
    state = "present",
})

caravel.core.file({
    src = "nginx.conf", -- stored with the manifest
    dest = "/etc/nginx/nginx.conf",
    mode = "0644"
})

caravel.core.service({
    name = "nginx",
    state = "running",
})
```

Now we can spice it up by having the service restart if the file changes:

```lua
caravel.core.package({
    name = "nginx",
    state = "present",
})

local nginx_conf = caravel.core.file({
    src = "nginx.conf", -- stored with the manifest
    dest = "/etc/nginx/nginx.conf",
    mode = "0644"
})

caravel.core.service({
    name = "nginx",
    state = "running",
    subscribe = { nginx_conf },
})
```

What if we need a custom restart handler?

```lua
caravel.core.package({
    name = "nginx",
    state = "present",
})

local nginx_conf = caravel.core.file({
    src = "nginx.conf", -- stored with the manifest
    dest = "/etc/nginx/nginx.conf",
    mode = "0644"
})

local custom_nginx_restarter = function()
    local check_conf = caravel.core.shell({
        cmd = "/usr/local/bin/check_nginx_conf.sh"
    })
    if check_conf.success then
        caravel.core.service({
            name = "nginx",
            state = "restarted",
        })
    end
end

caravel.core.service({
    name = "nginx",
    state = "running",
    subscribe = { 
        { nginx_conf, handler = custom_nginx_restarter }, 
    },
})
```


<sup><sub><a href="https://www.vecteezy.com/free-vector/caravel">Caravel Vectors by Vecteezy</a></sub></sup>
