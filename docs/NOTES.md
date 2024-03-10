# Notes

## Naming

The caravel is a small maneuverable sailing ship used in the 15th century by
the Portuguese to explore along the West African coast and into the
Atlantic Ocean. The lateen sails gave it speed and the capacity for
sailing windward. Caravels were used by the Portuguese and Castilians
for the oceanic exploration voyages during the 15th and 16th centuries,
during the Age of Discovery.

Deploy command called `ship`?

Lua configurations called `manifest`s.

I don't know how over-the-top I want to be with nautical-themed terminology
but it's fun!

## Lua

Actual implementation of built in modules (file, package, service, user, etc)
are all in Rust and exposed via the `core` namespace:

```lua
caravel.core.package({
    state = "present", -- or latest etc
    name = "nginx",
})
```

Because we're using lua, we don't need to prescribe any specific file layout.
Lua should just be able to import files without issue.
If this is not the case, we can figure out a file hierarchy.

A super important part of this project is the ability for users to store valid
caravel modules in a specific place
(`~/.caravel/modules/`, or `CARAVEL_MODULE_PATH`), then have caravel try to
auto-load those modules into its `extra` namespace:

```lua
caravel.extra.mysql_user({
    state = "present",
    name = "dbuser",
})
caravel.extra.mysql_grant({
    state = "present",
    name = "dbuser",
    database = "bigdata",
    permission = "*.*",
})
```

## Hiera/Variables

Both Puppet and Ansible use separate variable files from the manifests.
We could probably do some sort of specific file hierarchy as well, but again,
it can be flexible because lua.

## Examples of usage

These are just ideas

`caravel send manifest.lua --targets host1,host2` -- send to remote agent

`caravel send manifest.lua --groups group1,group2` -- using inventory system

`caravel send manifest.lua --targets localhost,host1` -- can apply locally too

This is the bolt way. How can we also facilitate the ansible way where the
manifest contains the information for which hosts it's applying to?

Maybe the manifest itself has an optional connection table where you can
configure things like groups, hosts, etc?

```lua
-- Specify the targets in the manifest itself
-- This will apply to all hosts in "mygroup1",
-- as well as "host1", and "host2"
caravel.setup.connection({
    groups = { "mygroup1" },
    hosts = { "host1", "host2" }
    timeout = 60,
})

caravel.core.file({
    path = "/etc/myservice.conf",
    content = "do something",
    owner = "root",
    group = "root",
    mode = "0600",
})
```

## Architecture

Caravel is an agent-based configuration management tool.
It supports both push and pull.
It would be cool if we could have the client and agent be the same codebase
and just behave differently based on the settings and how it's called.

### Agent

Since we'll operate in both push and pull, we'll need a few components:

- Two main threads

  - HTTP server listening on a port for something like a POST upload request

  - Sleeper that wakes up every X minutes, set by config, then pulls a remote
    location, probably a simple GET request to a plaintext manifest, like
    ansible-pull. The question would be, if we do dynamic importing via lua,
    How can we make that work via GET request? Do we just require that the
    manifest be in a git repo? Or do we have a couple traversal plugins that
    understand if the domain is github, or some static fileserver, and it can
    "try" to crawl for the other files? This is sort of magic, but would be
    super cool! OR, do we have a third mode, "Server" that will
    host the configs? (ehhhh)

These two threads will spawn subtasks to do work.

Ideas for configuration:

```lua
caravel.agent.config({
    pull = {
        manifest_url = "https://github.com/lcrownover/mysite/manifests/myservice.lua",
        interval = 1800, -- 30min
        splay = 120, -- start +/- 2min to prevent stampeding herd (default 0)
    }
    push = {
        port = 1336,
        keys = {
            {
                label = "Vasco da Gama",
                key = "AAAAB3NzaC1yc2EAAAADAQABAAACAQCsvNlhPGJrjABYoR5M0jy0/eK+At4fv64LXRLPTPvUCu4AFeGHraQPQH/r7PoBoiNsiQzgVmPPJXFKuT0JDsdgdhqXfpKc9Nf5ayYBhi+rMsdzaTylZYmmpQSptKK+2RdTsTwUYknUwGUXUpJ6xgoELCMIOCA6ld16oHMHjr3U3Ft6/wR9RRXxdJg0cg2/HrcCCyZiwBZKclc8cjG2aLj7L0q2cnsVw0j+G35sE2Plo0cgfcY26pzKE5FBpZnDmSnG+7WGJ9EOUBZ3gtepP6RfAMkdW3r/2HDc3R690Podh9NDrZjjJ3QhMypAcXHiakOT9xOxpMze6ef8FV2uIkB6hfXJIMlKn2c875IGPlH4OjOLnPvwLwGzcewWpc2KzM7EKKAMeuYPtdlM9l6aMbr5tJDQL6Q2wxGs6b8kOOM8FpCJeLi5WuJzVgAMyhdnvLldxi1izvhxeeF1qZvZXcnqSfbegPV62jidbK0zcQZmHT+BI8VPldBq3A/18IKtIAjbQ962UMAiZoMtEHmnaDH5wJ9zAxD0j7mKYAilPDqW+up9g6gt8yo84bVJ/2BK8NlcgEqxclj/Kup7Sq45ybRRGYaZBTJcbezY0lZePFtskBMi0MtPfEiPf3xZNKAYBr2VhojZnjx8RAZvM6yfGMDu2BR3hYmmj/Tf2M6ghQi0FQ==",
            },
            {
                label = "Pedro Cabral",
                key = "AAAAB3NzaC1yc2EAAAADAQABAAACAQCsvNlhPGJrjABYoR5M0jy0/eK+At4fv64LXRLPTPvUCu4AFeGHraQPQH/r7PoBoiNsiQzgVmPPJXFKuT0JDsdgdhqXfpKc9Nf5ayYBhi+rMsdzaTylZYmmpQSptKK+2RdTsTwUYknUwGUXUpJ6xgoELCMIOCA6ld16oHMHjr3U3Ft6/wR9RRXxdJg0cg2/HrcCCyZiwBZKclc8cjG2aLj7L0q2cnsVw0j+G35sE2Plo0cgfcY26pzKE5FBpZnDmSnG+7WGJ9EOUBZ3gtepP6RfAMkdW3r/2HDc3R690Podh9NDrZjjJ3QhMypAcXHiakOT9xOxpMze6ef8FV2uIkB6hfXJIMlKn2c875IGPlH4OjOLnPvwLwGzcewWpc2KzM7EKKAMeuYPtdlM9l6aMbr5tJDQL6Q2wxGs6b8kOOM8FpCJeLi5WuJzVgAMyhdnvLldxi1izvhxeeF1qZvZXcnqSfbegPV62jidbK0zcQZmHT+BI8VPldBq3A/18IKtIAjbQ962UMAiZoMtEHmnaDH5wJ9zAxD0j7mKYAilPDqW+up9g6gt8yo84bVJ/2BK8NlcgEqxclj/Kup7Sq45ybRRGYaZBTJcbezY0lZePFtskBMi0MtPfEiPf3xZNKAYBr2VhojZnjx8RAZvM6yfGMDu2BR3hYmmj/Tf2M6ghQi0FQ==",
            },
        }
    }
})
```

### Client

The client mode is much easier. It's mostly about validation of the manifest
and sending the POST to the agent (or telling the agent to pull). Since we're
going to do manifest validation on the server as well, the client really is just
the CLI portion...

## Testing

We should have ROBUST-AF testing for core modules on multiple distributions
via Docker.

## Documentation

Each module should be able to auto-generate its docs from Rust's standard
docstring method. I don't know what that is but it's how the rust docs are
generated.
It should also have a compatibility matrix for each OS with like little
red X's or green checkmarks.

## Security

Security between client and agent should be simple. I don't think we need a
full-blown CA like puppet uses. Maybe just having the agent do pub/priv key
validation would be enough? The agent could be configured with 1+ public keys,
allowing for a single service account style deployment, or each user has their
own key.

## Bootstrapping

It should be able to bootstrap itself via SSH in addition to curl-style setup.

`caravel bootstrap user@192.168.1.10 --config agent-config.lua`

Assuming you have SSH access to that host, it should be able to transfer itself
and set everything up to use the `agent-config.lua`.

## Manifest Compilation

I think it'd be best if whatever is sending the manifest does the compilation,
that way we're just sending one big hunk of data.

Compilation should include the following steps:

- Resolving imports from other Lua files, combining into one large Lua file

- Validating each resource (module)

- Grabbing all the source files (copying files and whatnot)

Should deliver some giant payload with everything to agent.

## Validation of manifest

Each module should have a `validate()` method that will make sure that the
correct parameters are set for a functional application of the module.
For example, ensuring that the default parameters are set, or that two
conflicting parameters aren't both set.

Validation should occur on the agent, regardless of mode. That way, module
resolution will always be accurate, and it's less heavy on the client (laptop).
