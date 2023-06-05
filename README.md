# **D**iscord **I**PC **P**roxy
> Discord Rich Presence (and other features) without having Discord on!

# Why?

I occasionally run memory intensive applications on my computer. Unfortunately, I often can't run Discord simultaneously
with these application without the risk of running out of memory. However, I still want my friends to be aware of what 
I'm currently doing or playing. Since I have a second computer it *would* be theoretically possible to redirect or proxy
all the requests sent to the IPC socket to my computer. I tried looking online to see if someone has already implemented
something akin to this project, but, alas, I've found none. Due to this, I decided to develop my own.

# Getting Started

## Prerequisites

* A second computer capable of running a Discord client (not the website).

## Instructions

1. Go to releases and download the package meant for your operating system.
    * There are two binaries packaged for each operating system, the host and remote. Here are explanations for what
      they mean:
      * **Host**: This is meant to be run on the computer which isn't running Discord. It will forward all requests to 
                  the remote, and all responses from the remote to the socket.
      * **Remote**: This is meant to be run on the computer which is running Discord. It will forward all requests from
                    the host to the Discord IPC, and all responses from Discord IPC to the host.
2. Link up your host and remote.
   * First, ensure that the two computers are on the same local area network.
   * Then, launch `dip_remote` on your remote. It should tell you the remote address (both IPv4 and IPv6). Try the IPv4
     address first, and if it doesn't work, use the IPv6 address. Make sure Discord is open!
   * Launch `dip_host` on your host, then pass the remote address onto the host. There are two ways to do this:
     * Pass it as an argument like so: `dip_host -r REMOTE_ADDRESS` or `dip_host --remote-address REMOTE_ADDRESS`.
       Here's an example:
       ```bash
       $ dip_host -r 192.168.86.31 # if the remote address uses the default port 49131, it can be elided
       $ dip_host --remote-address 192.168.86.31:20800
       ```
     * Through a configuration file. See the section below on how to configure the host and remote.
3. You're all good to go!

## Configuration

The configuration resolves in the following order: arguments passed, configuration files.

Here are the locations of `host.toml` and `remote.toml` depending on operating system.

* Linux
  * **host.toml**: `$XDG_CONFIG_HOME/dip/host.toml` or `$HOME/.config/dip/host.toml`, example is 
                     `/home/alp/.config/dip/host.toml`
  * **remote.toml**: `$XDG_CONFIG_HOME/dip/remote.toml` or `$HOME/.config/dip/remote.toml`, example is
                     `/home/alp/.config/dip/remote.toml`
* Mac OS
    * **host.toml**: `$HOME/Library/Application Support/ALinuxPerson.DIP/host.toml`, example is
      `/Users/AMacPerson/Library/Application Support/ALinuxPerson.DIP/host.toml`
    * **remote.toml**: `$HOME/Library/Application Support/ALinuxPerson.DIP/remote.toml`, example is
      `/Users/AMacPerson/Library/Application Support/ALinuxPerson.DIP/remote.toml`
* Windows
    * **host.toml**: `{FOLDERID_RoamingAppData}\ALinuxPerson\DIP\config\host.toml`, example is
                     `C:\Users\AWindowsPerson\AppData\Roaming\ALinuxPerson\DIP\config\host.toml`
    * **remote.toml**: `{FOLDERID_RoamingAppData}\ALinuxPerson\DIP\config\remote.toml`, example is
                     `C:\Users\AWindowsPerson\AppData\Roaming\ALinuxPerson\DIP\config\remote.toml`

Example configurations of `host.toml` and `remote.toml` can be found here: [host.toml](host/host.toml), [remote.toml](remote/remote.toml)

# Usage

`dip_remote` should be launched first on the remote computer, then `dip_host` can be launched on the host computer.

## Host

Launch `dip_host` with remote address passed in either the arguments (through -r / --remote-address) or configuration
file. It must match the address given by `dip_remote`. 

`dip_host` will tell you what remote address will be used to connect. See this sample log output to find out where it 
is:

```
2023-06-05T13:29:26.121833Z  INFO dip_host: socket path is /run/user/1000/discord-ipc-0
2023-06-05T13:29:26.121846Z  INFO dip_host: remote address to connect to remote_address=192.168.86.32:49131 // <-- this is the remote address
2023-06-05T13:29:26.121851Z  INFO dip_host: successfully resolved configuration
```

## Remote

Launch `dip_remote`. It should not take any arguments. If you want to set a port, you can pass the -p or --port 
argument.

Upon launch, `dip_remote` should tell you the remote address which will be used by `dip_host`. It will be given in IPv4,
and IPv6 if possible. Use the IPv4 address first, then use IPv6. See this sample log output to find out where it is:

```
2023-06-05T13:32:21.774508Z  INFO dip_remote: socket path is /run/user/1000/discord-ipc-0
2023-06-05T13:32:21.774534Z  INFO dip_remote: port to listen on port=49131
2023-06-05T13:32:21.774540Z  INFO dip_remote: successfully resolved configuration
2023-06-05T13:32:21.774694Z  INFO dip_remote: remote ipv4 address is 192.168.86.32:49131 // <---------- use these two
2023-06-05T13:32:21.774744Z  INFO dip_remote: remote ipv6 address is [::ffff:192.168.86.32]:49131 // <- addresses
```

# Terminologies

**Host Computer**: The computer which will run the host binary.

**Remote Computer**: The computer which will run the remote binary.

# License

Distributed under the MIT license. See [`LICENSE`](LICENSE) for more information.